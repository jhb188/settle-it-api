defmodule SettleIt.GameServer.PhysicsSimulator do
  use GenStage

  alias SettleIt.GameServer.Engine

  @physics_steps_per_second 60
  @refresh_interval round(1000 / @physics_steps_per_second)

  def start_link(game_id),
    do:
      GenStage.start_link(__MODULE__, game_id,
        name: String.to_atom("physics_simulator_" <> game_id)
      )

  @impl true
  def init(game_id) do
    Process.send_after(self(), :step, @refresh_interval)
    {:producer_consumer, nil, subscribe_to: [{String.to_atom("state_producer_" <> game_id), []}]}
  end

  @impl true
  def handle_info(:step, nil) do
    Process.send_after(self(), :step, @refresh_interval)

    {:noreply, [], nil}
  end

  @impl true
  def handle_info(:step, state) do
    step_start = :os.system_time(:millisecond)
    next_game_state = Engine.step(state)

    GenServer.cast(
      String.to_existing_atom("state_producer_" <> next_game_state.id),
      {:new_state, next_game_state}
    )

    step_time_elapsed = :os.system_time(:millisecond) - step_start

    refresh_interval =
      case @refresh_interval - step_time_elapsed do
        next_interval when next_interval > 0 -> next_interval
        _otherwise -> @refresh_interval
      end

    Process.send_after(self(), :step, refresh_interval)

    {:noreply, [next_game_state], next_game_state}
  end

  @impl true
  def handle_events(game_states, _from, state) do
    next_state =
      case List.last(game_states) do
        nil -> state
        new_state -> new_state
      end

    {:noreply, [], next_state}
  end
end
