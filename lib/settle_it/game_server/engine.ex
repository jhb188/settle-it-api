defmodule SettleIt.GameServer.Engine do
  alias SettleIt.GameServer.State
  alias SettleIt.GameServer.Physics

  @type player_id :: String.t()
  @type coordinate :: non_neg_integer()

  @player_distance_from_center 50.0
  @player_height 1.0
  @player_mass 100.0
  @bullet_mass 0.05
  @team_colors [
    "red",
    "orange",
    "yellow",
    "green",
    "blue",
    "purple",
    "brown",
    "light-red",
    "light-orange",
    "light-yellow",
    "light-green",
    "light-blue",
    "light-purple",
    "light-brown",
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
  def start(%State.Game{} = state) do
    players = Map.values(state.players)
    player_bodies = get_spaced_player_bodies(players)
    world_bodies = Physics.init_world()

    %State.Game{state | status: :playing, bodies: Map.merge(world_bodies, player_bodies)}
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
    next_bodies =
      Map.update!(bodies, player_id, fn body ->
        # do not allow move requests to reposition player_height
        {_current_x, _current_y, current_z} = body.translation
        %State.Body{body | translation: {x / 1, y / 1, current_z}}
      end)

    %State.Game{state | bodies: next_bodies}
  end

  def rotate_player(%State.Game{bodies: bodies} = state, player_id, angle) do
    next_bodies =
      Map.update!(bodies, player_id, fn body ->
        %State.Body{body | rotation: {0.0, 0.0, angle / 1}}
      end)

    %State.Game{state | bodies: next_bodies}
  end

  def jump_player(%State.Game{bodies: bodies} = state, player_id) do
    next_bodies = Map.update!(bodies, player_id, &Physics.apply_jump/1)

    %State.Game{state | bodies: next_bodies}
  end

  def add_bullet(
        %State.Game{bodies: bodies, players: players} = state,
        player_id,
        position,
        linvel
      ) do
    bullet_id = UUID.uuid4()

    bullet = %State.Body{
      id: bullet_id,
      owner_id: player_id,
      team_id: players[player_id].team_id,
      translation: {position.x, position.y, position.z},
      linvel: {linvel.x / 1, linvel.y / 1, linvel.z / 1},
      rotation: {0.0, 0.0, 0.0},
      mass: @bullet_mass,
      class: :bullet
    }

    %State.Game{state | bodies: Map.put(bodies, bullet_id, bullet)}
  end

  def step(%State.Game{last_updated: last_updated, bodies: bodies} = state) do
    target_time = :os.system_time(:millisecond)
    dt = target_time - last_updated
    dt_seconds = dt / 1000

    {updated_bodies, won?} = Physics.step(bodies, dt_seconds)

    next_state = %State.Game{
      state
      | bodies: updated_bodies,
        last_updated: target_time
    }

    if won? do
      %State.Game{next_state | status: :finished}
    else
      next_state
    end
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
       %State.Body{
         id: player.id,
         team_id: player.team_id,
         translation: {x, y, z},
         rotation: {0.0, 0.0, rotation},
         mass: @player_mass,
         class: :player,
         hp: 10
       }}
    end)
    |> Map.new()
  end

  defp get_unused_team_color(%State.Game{teams: teams}) do
    used_colors = Enum.map(teams, fn {_team_id, team} -> team.color end)

    team_colors =
      if teams |> Map.values() |> length() |> odd? do
        Enum.reverse(@team_colors)
      else
        @team_colors
      end

    Enum.find(team_colors, "red", fn color -> not Enum.member?(used_colors, color) end)
  end

  defp odd?(n), do: rem(n, 2) == 1
end
