defmodule SettleIt.GameServer do
  @moduledoc """
  Runs the game
  """
  use GenServer

  alias SettleIt.GameServer.Engine
  alias SettleIt.GameServer.State

  @physics_steps_per_second 60
  @refresh_interval round(1000 / @physics_steps_per_second)

  @impl true
  def init(_game_id) do
    Process.send_after(self(), :step, 10)

    {:ok, Engine.init()}
  end

  def start_link(game_id) do
    GenServer.start_link(__MODULE__, game_id, name: String.to_atom(game_id))
  end

  @impl true
  def handle_call({:player_join, player, pid}, _from, state) do
    next_game_state = Engine.add_player(state, player, pid)

    notify_subscribers(next_game_state, {:game_updated, next_game_state})

    {:reply, next_game_state, next_game_state}
  end

  @impl true
  def handle_cast({:player_leave, player_id}, state) do
    next_game_state = Engine.remove_player(state, player_id)

    cond do
      State.Game.empty?(next_game_state) ->
        kill_game_server()
        {:noreply, next_game_state}

      true ->
        notify_subscribers(next_game_state, {:game_updated, next_game_state})
        {:noreply, next_game_state}
    end
  end

  @impl true
  def handle_cast(
        {:player_move, player_id, coords},
        state
      ) do
    next_game_state = Engine.move_player(state, player_id, coords)

    {:noreply, next_game_state}
  end

  @impl true
  def handle_cast(
        {:player_rotate, player_id, angle},
        state
      ) do
    next_game_state = Engine.rotate_player(state, player_id, angle)

    {:noreply, next_game_state}
  end

  @impl true
  def handle_cast(
        {:player_jump, player_id},
        state
      ) do
    next_game_state = Engine.jump_player(state, player_id)

    {:noreply, next_game_state}
  end

  @impl true
  def handle_cast(
        {:player_shoot, player_id, position, linvel},
        state
      ) do
    next_game_state = Engine.add_bullet(state, player_id, position, linvel)

    {:noreply, next_game_state}
  end

  @impl true
  def handle_cast(:start_game, state) do
    next_game_state = Engine.start(state)
    notify_subscribers(next_game_state, {:game_updated, next_game_state})
    {:noreply, next_game_state}
  end

  @impl true
  def handle_cast(:restart_game, state) do
    next_game_state = Engine.restart(state)
    notify_subscribers(next_game_state, {:game_updated, next_game_state})
    {:noreply, next_game_state}
  end

  @impl true
  def handle_info(:step, state) do
    step_start = :os.system_time(:millisecond)
    next_game_state = Engine.step(state)
    step_time_elapsed = :os.system_time(:millisecond) - step_start

    refresh_interval =
      case @refresh_interval - step_time_elapsed do
        next_interval when next_interval > 0 -> next_interval
        _otherwise -> @refresh_interval
      end

    Process.send_after(self(), :step, refresh_interval)

    notify_subscribers(next_game_state, {:game_updated, next_game_state})

    {:noreply, next_game_state}
  end

  defp notify_subscribers(game_state, message) do
    game_state
    |> State.Game.get_subscribed_processes()
    |> Enum.each(fn pid -> Process.send(pid, message, []) end)

    :ok
  end

  defp kill_game_server() do
    Process.exit(self(), :kill)
  end
end
