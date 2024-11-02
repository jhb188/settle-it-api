defmodule SettleIt.GameServer.Engine do
  alias SettleIt.GameServer.State

  @type player_id :: String.t()
  @type coordinate :: non_neg_integer()

  @player_distance_from_center 50.0
  @player_height 2.0
  @player_mass 100.0
  @bullet_mass 0.05
  @bullet_size 0.10
  @team_colors [
    "light-red",
    "light-orange",
    "light-yellow",
    "light-green",
    "light-blue",
    "light-purple",
    "light-brown",
    "red",
    "orange",
    "yellow",
    "green",
    "blue",
    "purple",
    "brown",
    "dark-red",
    "dark-orange",
    "dark-yellow",
    "dark-green",
    "dark-blue",
    "dark-purple",
    "dark-brown"
  ]

  @doc """
  Initializes a State.Game
  """
  def init(game_id) do
    %State.Game{
      id: game_id,
      last_updated: :os.system_time(:millisecond)
    }
  end

  @doc """
  Starts the game
  """
  def start(%State.Game{} = state, physics_port) do
    players = Map.values(state.players)

    player_bodies = get_spaced_player_bodies(players)

    state = %State.Game{
      state
      | physics_port: physics_port
    }

    Enum.each(player_bodies, fn {_body_id, body} -> add_player(state, body) end)

    state
  end

  @doc """
  Restarts a State.Game
  """
  def restart(%State.Game{} = _state) do
    %State.Game{}
  end

  def update_topic(%State.Game{} = state, topic) do
    %State.Game{state | topic: topic}
  end

  def create_team(%State.Game{} = state, owner_id, cause) do
    existing_team_causes = state.teams |> Map.values() |> Enum.map(& &1.cause)

    if Enum.member?(existing_team_causes, cause) do
      state
    else
      new_team = %State.Team{
        id: UUID.uuid4(),
        owner_id: owner_id,
        cause: cause,
        color: get_unused_team_color(state)
      }

      move_player_to_team(
        %State.Game{state | teams: Map.put(state.teams, new_team.id, new_team)},
        owner_id,
        new_team.id
      )
    end
  end

  def delete_team(%State.Game{teams: teams} = state, team_id) do
    %State.Game{state | teams: Map.delete(teams, team_id)}
  end

  def move_player_to_team(%State.Game{players: players} = state, player_id, team_id) do
    player = players[player_id]
    next_players = Map.put(players, player_id, %State.Player{player | team_id: team_id})

    %State.Game{state | players: next_players}
  end

  @doc """
  Adds a player to a State.Game
  """
  def add_player(
        %State.Game{players: players} = state,
        player,
        pid
      ) do
    next_players =
      Map.put(players, player.id, %State.Player{
        name: player.name,
        id: player.id,
        pid: pid
      })

    %State.Game{state | players: next_players}
  end

  @doc """
  Removes a player from a State.Game by player_id
  """
  def remove_player(
        %State.Game{players: players} = state,
        player_id
      ) do
    next_players = Map.delete(players, player_id)

    %State.Game{state | players: next_players}
  end

  def update_player_name(%State.Game{players: players} = state, player_id, name) do
    next_players =
      Map.update!(players, player_id, fn player -> %State.Player{player | name: name} end)

    %State.Game{state | players: next_players}
  end

  def move_player(%State.Game{bodies: bodies} = state, player_id, %{
        x: x,
        y: y
      }) do
    body = Map.get(bodies, player_id)

    msg = %{action: "move", x: x, y: y, id: body.id}

    send_physics_message(
      state,
      msg
    )

    state
  end

  def rotate_player(%State.Game{bodies: bodies} = state, player_id, angle) do
    body = Map.get(bodies, player_id)

    msg = %{action: "rotate", id: body.id, rotation_angle: angle / 1}
    send_physics_message(state, msg)

    state
  end

  def jump_player(%State.Game{bodies: bodies} = state, player_id) do
    body = bodies |> Map.get(player_id) |> apply_jump()
    {_linvelx, _linvely, linvelz} = body.linvel

    msg = %{action: "jump", id: body.id, linvel_z: linvelz}
    send_physics_message(state, msg)

    state
  end

  def add_bullet(
        %State.Game{players: players} = state,
        player_id,
        position,
        linvel
      ) do
    bullet_id = UUID.uuid4()

    bullet = %{
      id: bullet_id,
      owner_id: player_id,
      team_id: players[player_id].team_id,
      translation: {position.x, position.y, position.z},
      linvel: {linvel.x / 1, linvel.y / 1, linvel.z / 1},
      angvel: {0.0, 0.0, 0.0},
      dimensions: {@bullet_size, @bullet_size, @bullet_size},
      rotation: {0.0, 0.0, 0.0},
      mass: @bullet_mass,
      class: "bullet",
      hp: 0
    }

    msg = Map.put(bullet, :action, "shoot")
    send_physics_message(state, msg)

    state
  end

  defp get_spaced_player_bodies([]), do: %{}

  defp get_spaced_player_bodies(players) do
    num_players = length(players)
    circumference = 2 * :math.pi()
    angle_size = circumference / num_players

    players
    |> Enum.with_index()
    |> Enum.map(fn {player, i} ->
      current_angle = angle_size * i
      x = @player_distance_from_center * :math.cos(current_angle)
      y = @player_distance_from_center * :math.sin(current_angle)
      z = @player_height / 2

      # angle 0 corresponds to an orientation resting on the x axis, looking in the direction of
      # the y axis, with the z axis being directly up. rotating (2pi / 4) counterclockwise makes the
      # orientation face the origin
      rotation = Math.rad2deg(current_angle + circumference / 4)

      {player.id,
       %{
         id: player.id,
         team_id: player.team_id,
         owner_id: player.id,
         translation: {x, y, z},
         linvel: {0.0, 0.0, 0.0},
         angvel: {0.0, 0.0, 0.0},
         rotation: {0.0, 0.0, rotation},
         dimensions: {0.0, 0.525, 2.0},
         mass: @player_mass,
         class: "player",
         hp: 10
       }}
    end)
    |> Map.new()
  end

  defp get_unused_team_color(%State.Game{teams: teams}) do
    used_colors = Enum.map(teams, fn {_team_id, team} -> team.color end)

    Enum.find(@team_colors, "red", fn color -> not Enum.member?(used_colors, color) end)
  end

  defp send_physics_message(%State.Game{physics_port: port}, msg) do
    Port.command(port, Jason.encode!(msg) <> "\n")
  end

  defp add_player(state, body) do
    msg = Map.put(body, :action, "add_player")
    send_physics_message(state, msg)
  end

  defp apply_jump(player_body) do
    if can_jump?(player_body) do
      {linvelx, linvely, _linvelz} = player_body.linvel

      %{player_body | linvel: {linvelx, linvely, 10.0}}
    else
      player_body
    end
  end

  defp can_jump?(%{linvel: {_x, _y, linvelz}}) when linvelz < 0.0001 and linvelz > -0.0001,
    do: true

  defp can_jump?(_body), do: false
end
