defmodule SettleIt.GameServer.StateProducer do
  use GenStage

  alias SettleIt.GameServer.Engine
  alias SettleIt.GameServer.State

  @physics_steps_per_second 60
  @refresh_interval round(1000 / @physics_steps_per_second)

  def start_link(game_id),
    do:
      GenStage.start_link(__MODULE__, game_id, name: String.to_atom("state_producer_" <> game_id))

  @impl true
  def init(game_id) do
    {:producer_consumer, Engine.init(game_id),
     subscribe_to: [{String.to_existing_atom(game_id), interval: @refresh_interval}]}
  end

  @impl true
  def handle_events(actions, _from, %State.Game{status: :pending} = state) do
    next_state = Enum.reduce(actions, state, &apply_action/2)

    {:noreply, [{:state_update, next_state}], next_state}
  end

  @impl true
  def handle_events(actions, _from, state) do
    next_state = Enum.reduce(actions, state, &apply_action/2)

    {:noreply, [], next_state}
  end

  @impl true
  def handle_info(:step, state) do
    next_game_state = Engine.step(state)

    physics_execution_time = :os.system_time(:millisecond) - next_game_state.last_updated

    event =
      case next_game_state.status do
        :playing ->
          next_physics_step_time =
            if physics_execution_time > @refresh_interval do
              0
            else
              @refresh_interval - physics_execution_time
            end

          Process.send_after(self(), :step, next_physics_step_time)

          {:bodies_update, next_game_state}

        :finished ->
          {:state_update, next_game_state}
      end

    {:noreply, [event], next_game_state}
  end

  @impl true
  def handle_info(:kill_if_empty, %State.Game{players: players} = state) when players == %{} do
    Process.exit(self(), :kill)

    {:noreply, [], state}
  end

  @impl true
  def handle_info(:kill_if_empty, state) do
    {:noreply, [], state}
  end

  defp apply_action({:player_start_lobby, player, pid, topic}, state) do
    state
    |> Engine.add_player(player, pid)
    |> Engine.update_topic(topic)
  end

  defp apply_action({:player_join, player, pid}, state), do: Engine.add_player(state, player, pid)

  defp apply_action({:player_leave, player_id}, state) do
    case Engine.remove_player(state, player_id) do
      %State.Game{players: players} when players == %{} ->
        Process.send_after(self(), :kill_if_empty, 10_000)
        state

      state ->
        state
    end
  end

  defp apply_action({:player_update_name, player_id, name}, state) do
    Engine.update_player_name(state, player_id, name)
  end

  defp apply_action({:player_join_team, player_id, team_id}, state) do
    Engine.move_player_to_team(state, player_id, team_id)
  end

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

  defp apply_action({:create_team, owner_id, team_cause}, state) do
    Engine.create_team(state, owner_id, team_cause)
  end

  defp apply_action({:delete_team, team_id}, state) do
    Engine.delete_team(state, team_id)
  end

  defp apply_action(:start_game, state) do
    Process.send_after(self(), :step, @refresh_interval)

    Engine.start(state)
  end

  defp apply_action(:restart_game, state) do
    Engine.restart(state)
  end

  defp apply_action(_action, state), do: state
end
