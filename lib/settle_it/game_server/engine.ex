defmodule SettleIt.GameServer.Engine do
  alias SettleIt.GameServer.State
  alias SettleIt.GameServer.Physics

  @type player_id :: String.t()
  @type coordinate :: non_neg_integer()

  @doc """
  Initializes a State.Game
  """
  def init(), do: %State.Game{}

  @spec start(%SettleIt.GameServer.State.Game{:status => any, optional(any) => any}) ::
          %SettleIt.GameServer.State.Game{:status => :started, optional(any) => any}
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
        :pending -> Physics.add_player(bodies, player.id)
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
        :pending -> Physics.remove_player(bodies, player_id)
        _ -> bodies
      end

    %State.Game{state | players: next_players, bodies: next_bodies}
  end

  def move_player(%State.Game{bodies: bodies} = state, player_id, %{
        x: x,
        y: y
      }) do
    next_bodies =
      Enum.map(bodies, fn body ->
        if body.id == player_id do
          # do not allow move requests to reposition height
          {_current_x, _current_y, current_z} = body.translation
          %Physics.Body{body | translation: {y / 1, x / 1, current_z}}
        else
          body
        end
      end)

    %State.Game{state | bodies: next_bodies}
  end

  def jump_player(%State.Game{bodies: bodies} = state, player_id) do
    next_bodies =
      Enum.map(bodies, fn body ->
        if body.id == player_id do
          Physics.apply_jump(body)
        else
          body
        end
      end)

    %State.Game{state | bodies: next_bodies}
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

  @doc """
  Starts the next round
  """
  def next_round(%State.Game{}) do
  end
end
