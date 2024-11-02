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
    # if multiple game_states have been sent, we only need the most recent one
    game_states |> List.last() |> handle_event()

    {:noreply, [], state}
  end

  defp handle_event({:state_update, %State.Game{} = state}) do
    game_update = GameUpdate.from_state(state)

    notify_subscribers(state, {:game_updated, game_update})
  end

  defp handle_event({:bodies_update, %State.Game{} = state}) do
    bodies_update = GameUpdate.bodies_update_from_state(state)

    notify_subscribers(state, {:bodies_updated, bodies_update})
  end

  defp notify_subscribers(game_state, message) do
    game_state
    |> State.Game.get_subscribed_processes()
    |> Enum.each(fn pid -> Process.send(pid, message, []) end)
  end
end
