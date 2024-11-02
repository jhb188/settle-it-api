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
  def handle_events(actions, _from, state) do
    {next_state, events} = apply_actions(actions, state)

    {:noreply, events, next_state}
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

  @impl true
  def handle_info({_port, {:data, data}}, state) do
    case decode_physics(state, data) do
      :game_won ->
        next_game_state = %{state | status: :finished}
        {:noreply, [{:state_update, next_game_state}], next_game_state}

      {new_bodies, extra_data} ->
        new_world_bodies =
          new_bodies |> Enum.filter(fn {_k, body} -> body.class == "obstacle" end) |> Map.new()

        world_bodies =
          if new_world_bodies == %{} do
            state.world_bodies
          else
            new_world_bodies
          end

        other_bodies =
          new_bodies |> Enum.reject(fn {_k, body} -> body.class == "obstacle" end) |> Map.new()

        next_bodies =
          state.bodies
          |> Enum.reject(fn {_k, body} -> body.class == "bullet" end)
          |> Map.new()
          |> Map.merge(other_bodies)

        status =
          if new_world_bodies !== %{} do
            :playing
          else
            state.status
          end

        next_game_state = %{
          state
          | bodies: next_bodies,
            world_bodies: world_bodies,
            status: status,
            last_updated: :os.system_time(:millisecond),
            physics_data: extra_data
        }

        events =
          cond do
            state.status !== next_game_state.status ->
              [{:state_update, next_game_state}]

            new_bodies !== %{} ->
              [{:bodies_update, next_game_state}]

            true ->
              []
          end

        {:noreply, events, next_game_state}
    end
  end

  def handle_info({:EXIT, _port, reason}, state) do
    IO.inspect("the port closed unexpectedly: #{reason}")

    {:noreply, [], state}
  end

  defp apply_actions(actions, state), do: do_apply_actions(actions, {state, []})

  defp do_apply_actions([], accum) do
    accum
  end

  defp do_apply_actions([action | rest], {state, events}) do
    {next_state, event} = apply_action(action, state)

    next_events =
      case event do
        nil -> events
        event -> events ++ [event]
      end

    do_apply_actions(rest, {next_state, next_events})
  end

  defp apply_action({:player_start_lobby, player, pid, topic}, state) do
    next_state =
      state
      |> Engine.add_player(player, pid)
      |> Engine.update_topic(topic)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action({:player_join, player, pid}, state) do
    next_state = Engine.add_player(state, player, pid)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action({:player_leave, player_id}, %{status: :pending} = state) do
    case Engine.remove_player(state, player_id) do
      %State.Game{players: players} = state when players == %{} ->
        Process.send_after(self(), :kill_if_empty, 10_000)
        {state, state_update_msg(state)}

      state ->
        {state, state_update_msg(state)}
    end
  end

  defp apply_action({:player_leave, _player_id}, state) do
    {state, nil}
  end

  defp apply_action({:player_update_name, player_id, name}, state) do
    next_state = Engine.update_player_name(state, player_id, name)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action({:player_join_team, player_id, team_id}, state) do
    next_state = Engine.move_player_to_team(state, player_id, team_id)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action(
         {:player_move, player_id, coords},
         state
       ) do
    {Engine.move_player(state, player_id, coords), nil}
  end

  defp apply_action(
         {:player_rotate, player_id, angle},
         state
       ) do
    {Engine.rotate_player(state, player_id, angle), nil}
  end

  defp apply_action(
         {:player_jump, player_id},
         state
       ) do
    {Engine.jump_player(state, player_id), nil}
  end

  defp apply_action(
         {:player_shoot, player_id, position, linvel},
         state
       ) do
    {Engine.add_bullet(state, player_id, position, linvel), nil}
  end

  defp apply_action({:create_team, owner_id, team_cause}, state) do
    next_state = Engine.create_team(state, owner_id, team_cause)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action({:delete_team, team_id}, state) do
    next_state = Engine.delete_team(state, team_id)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action(:start_game, state) do
    physics_port = Port.open({:spawn_executable, physics_executable()}, [:binary])

    next_state = Engine.start(state, physics_port)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action(:restart_game, state) do
    next_state = Engine.restart(state)

    {next_state, state_update_msg(next_state)}
  end

  defp apply_action(_action, state), do: {state, nil}

  defp physics_executable do
    mix_dir = Mix.env() |> mix_dir()
    phys_dir = "RUST_ENV" |> System.get_env() |> physics_dir()

    "_build/#{mix_dir}/rustler_crates/physics/#{phys_dir}/physics"
  end

  defp mix_dir("prod"), do: "prod"
  defp mix_dir(:prod), do: "prod"
  defp mix_dir(_), do: "dev"

  defp physics_dir("prod"), do: "release"
  defp physics_dir(:prod), do: "release"
  defp physics_dir(_), do: "debug"

  defp decode_physics(state, raw) do
    raw = state.physics_data <> raw

    Engine.Message.decode(raw)
  end

  defp state_update_msg(state) do
    {:state_update, state}
  end
end
