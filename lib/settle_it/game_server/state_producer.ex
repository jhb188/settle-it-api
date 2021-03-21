defmodule SettleIt.GameServer.StateProducer do
  use GenStage

  alias SettleIt.GameServer.Engine

  @physics_steps_per_second 60
  @refresh_interval round(1000 / @physics_steps_per_second)

  def start_link(game_id),
    do:
      GenStage.start_link(__MODULE__, game_id, name: String.to_atom("state_producer_" <> game_id))

  @impl true
  def init(game_id) do
    Process.send_after(self(), :step, @refresh_interval)

    {:producer_consumer, Engine.init(game_id),
     subscribe_to: [{String.to_existing_atom(game_id), max_demand: 100}]}
  end

  @impl true
  def handle_events(actions, _from, state) do
    next_state = Enum.reduce(actions, state, &apply_action/2)

    {:noreply, [], next_state}
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

    {:noreply, [next_game_state], next_game_state}
  end

  defp apply_action({:player_join, player, pid}, state), do: Engine.add_player(state, player, pid)

  defp apply_action({:player_leave, player_id}, state), do: Engine.remove_player(state, player_id)

  defp apply_action(
         {:player_move, player_id, coords},
         state
       ) do
    Engine.move_player(state, player_id, coords)
  end

  defp apply_action(
         {:player_rotate, player_id, angle},
         state
       ) do
    Engine.rotate_player(state, player_id, angle)
  end

  defp apply_action(
         {:player_jump, player_id},
         state
       ) do
    Engine.jump_player(state, player_id)
  end

  defp apply_action(
         {:player_shoot, player_id, position, linvel},
         state
       ) do
    Engine.add_bullet(state, player_id, position, linvel)
  end

  defp apply_action(:start_game, state) do
    Engine.start(state)
  end

  defp apply_action(:restart_game, state) do
    Engine.restart(state)
  end

  defp apply_action(_action, state), do: state
end
