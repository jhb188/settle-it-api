defmodule SettleIt.GameServer.NotificationsDispatcher do
  use GenStage

  alias SettleIt.GameServer.State
  alias SettleIt.GameServer.Notifications.GameUpdate

  def start_link(game_id),
    do:
      GenStage.start_link(__MODULE__, game_id,
        name: String.to_atom("notifications_dispatcher_" <> game_id)
      )

  def init(game_id) do
    {:consumer, :ok, subscribe_to: [{String.to_atom("state_producer_" <> game_id), []}]}
  end

  def handle_events([], _from, state) do
    {:no_reply, [], state}
  end

  def handle_events(game_states, _from, state) do
    game_states |> List.last() |> handle_event()

    {:noreply, [], state}
  end

  defp handle_event({:state_update, %State.Game{} = state}) do
    notify_subscribers_game_updated(state)
  end

  defp handle_event({:bodies_update, %State.Game{} = state}) do
    notify_subscribers_bodies_updated(state)
  end

  defp notify_subscribers(game_state, message) do
    game_state
    |> State.Game.get_subscribed_processes()
    |> Enum.each(fn pid -> Process.send(pid, message, []) end)
  end

  defp notify_subscribers_game_updated(game_state) do
    game_update = GameUpdate.from_state(game_state)

    notify_subscribers(game_state, {:game_updated, game_update})
  end

  defp notify_subscribers_bodies_updated(game_state) do
    bodies_update = GameUpdate.bodies_update_from_state(game_state)

    notify_subscribers(game_state, {:bodies_updated, bodies_update})
  end
end
