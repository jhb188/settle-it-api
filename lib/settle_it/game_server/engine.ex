defmodule SettleIt.GameServer.Engine do
  alias SettleIt.GameServer.State
  alias SettleIt.GameServer.Physics
  alias SettleIt.GameServer.Physics.Body

  @type player_id :: String.t()
  @type coordinate :: non_neg_integer()

  @player_distance_from_center 50.0
  @player_height 1.0
  @player_mass 100.0
  @bullet_mass 0.05

  @doc """
  Initializes a State.Game
  """
  def init(game_id) do
    %State.Game{
      id: game_id,
      last_updated: :os.system_time(:millisecond),
      bodies: Physics.init_world()
    }
  end

  @doc """
  Starts the game
  """
  def start(%State.Game{} = state) do
    %State.Game{state | status: :started}
  end

  @doc """
  Restarts a State.Game
  """
  def restart(%State.Game{} = _state) do
    %State.Game{}
  end

  @doc """
  Adds a player to a State.Game
  """
  def add_player(
        %State.Game{players: players, bodies: bodies, status: status} = state,
        player,
        pid
      ) do
    next_players =
      Map.put(players, player.id, %State.Player{
        name: player.name,
        id: player.id,
        pid: pid
      })

    next_bodies =
      case status do
        :pending -> do_add_player(bodies, player.id)
        _ -> bodies
      end

    %State.Game{state | players: next_players, bodies: next_bodies}
  end

  @doc """
  Removes a player from a State.Game by player_id
  """
  def remove_player(
        %State.Game{players: players, status: status, bodies: bodies} = state,
        player_id
      ) do
    next_players = Map.delete(players, player_id)

    next_bodies =
      case status do
        :pending -> do_remove_player(bodies, player_id)
        _ -> bodies
      end

    %State.Game{state | players: next_players, bodies: next_bodies}
  end

  def move_player(%State.Game{bodies: bodies} = state, player_id, %{
        x: x,
        y: y
      }) do
    next_bodies =
      Map.update!(bodies, player_id, fn body ->
        # do not allow move requests to reposition player_height
        {_current_x, _current_y, current_z} = body.translation
        %Physics.Body{body | translation: {x / 1, y / 1, current_z}}
      end)

    %State.Game{state | bodies: next_bodies}
  end

  def rotate_player(%State.Game{bodies: bodies} = state, player_id, angle) do
    next_bodies =
      Map.update!(bodies, player_id, fn body ->
        %Physics.Body{body | rotation: {0.0, 0.0, angle / 1}}
      end)

    %State.Game{state | bodies: next_bodies}
  end

  def jump_player(%State.Game{bodies: bodies} = state, player_id) do
    next_bodies = Map.update!(bodies, player_id, &Physics.apply_jump/1)

    %State.Game{state | bodies: next_bodies}
  end

  def add_bullet(%State.Game{bodies: bodies} = state, player_id, position, linvel) do
    bullet_id = UUID.uuid4()

    bullet = %Body{
      id: bullet_id,
      translation: {position.x, position.y, position.z},
      linvel: {linvel.x / 1, linvel.y / 1, linvel.z / 1},
      rotation: {0.0, 0.0, 0.0},
      mass: @bullet_mass,
      class: :bullet,
      owner_id: player_id
    }

    %State.Game{state | bodies: Map.put(bodies, bullet_id, bullet)}
  end

  def step(%State.Game{last_updated: last_updated, bodies: bodies} = state) do
    target_time = :os.system_time(:millisecond)
    dt = target_time - last_updated
    dt_seconds = dt / 1000

    updated_bodies = Physics.step(bodies, dt_seconds)

    %State.Game{
      state
      | bodies: updated_bodies,
        last_updated: target_time
    }
  end

  def do_add_player(bodies, new_player_id) do
    {players, nonplayers} = split_players_and_nonplayers(bodies)

    player_ids =
      [new_player_id | Map.keys(players)]
      |> Enum.uniq()

    Map.merge(nonplayers, get_spaced_player_bodies(player_ids))
  end

  def do_remove_player(bodies, player_id_to_remove) do
    {players, nonplayers} = split_players_and_nonplayers(bodies)

    player_ids = players |> Map.delete(player_id_to_remove) |> Map.keys()

    Map.merge(nonplayers, get_spaced_player_bodies(player_ids))
  end

  defp get_spaced_player_bodies([]), do: %{}

  defp get_spaced_player_bodies(player_ids) do
    num_players = length(player_ids)
    circumference = 2 * :math.pi()
    angle_size = circumference / num_players

    player_ids
    |> Enum.with_index()
    |> Enum.map(fn {player_id, i} ->
      current_angle = angle_size * i
      x = @player_distance_from_center * :math.cos(current_angle)
      y = @player_distance_from_center * :math.sin(current_angle)
      z = @player_height / 2

      # angle 0 corresponds to an orientation resting on the x axis, looking in the direction of
      # the y axis, with the z axis being directly up. rotating (2pi / 4) counterclockwise makes the
      # orientation face the origin
      rotation = Math.rad2deg(current_angle + circumference / 4)

      {player_id,
       %Body{
         id: player_id,
         translation: {x, y, z},
         rotation: {0.0, 0.0, rotation},
         mass: @player_mass,
         class: :player,
         hp: 10
       }}
    end)
    |> Map.new()
  end

  defp split_players_and_nonplayers(bodies) do
    {players, nonplayers} = Enum.split_with(bodies, fn {_id, body} -> body.class == :player end)

    {Map.new(players), Map.new(nonplayers)}
  end

  @doc """
  Starts the next round
  """
  def next_round(%State.Game{}) do
  end
end
