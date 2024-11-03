defmodule SettleItWeb.GameChannelTest do
  use SettleItWeb.ChannelCase, async: true
  alias SettleItWeb.GameChannel

  setup do
    {:ok, _, socket} =
      socket(SettleItWeb.UserSocket, nil, %{})
      |> subscribe_and_join(GameChannel, "game:123", %{"player_id" => "player1"})

    {:ok, socket: socket}
  end

  test "join with topic and player_id", %{socket: socket} do
    assert socket.assigns.player.id == "player1"
    assert socket.assigns.game_id == "123"
  end

  test "handle_info game_updated event", %{socket: socket} do
    send(socket.channel_pid, {:game_updated, %{status: "running"}})
    assert_push "game:updated", %{status: "running"}
  end

  test "handle_info bodies_updated event", %{socket: socket} do
    send(socket.channel_pid, {:bodies_updated, %{bodies: [%{id: "body1"}]}})
    assert_push "bodies:updated", %{bodies: [%{id: "body1"}]}
  end
end
