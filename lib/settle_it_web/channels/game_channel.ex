defmodule SettleItWeb.GameChannel do
  use SettleItWeb, :channel

  alias SettleIt.GameServer.Notifications.GameUpdate
  alias SettleIt.GameServer.State
  alias SettleIt.Players.Player

  def join("game:" <> game_id, id, socket) do
    with %Player{} = user <- %Player{name: "", id: id},
         socket <- init_socket(socket, game_id, user),
         %State.Game{} = game_state <- join_game_server(socket) do
      {:ok, GameUpdate.from_state(game_state), socket}
    else
      nil -> {:error, :not_found}
      error -> error
    end
  end

  def terminate(_reason, socket) do
    leave_game_server(socket)

    {:noreply, socket}
  end

  def handle_in("start_game", _params, socket) do
    notify_game_server_start_game(socket)

    {:noreply, socket}
  end

  def handle_in("restart_game", _game_id, socket) do
    notify_game_server_new_game(socket)

    {:noreply, socket}
  end

  def handle_in(
        "player_move",
        %{"player_id" => player_id, "x" => x, "y" => y},
        socket
      ) do
    notify_game_server_player_move(socket, player_id, %{x: x, y: y})

    {:noreply, socket}
  end

  def handle_in(
        "player_rotate",
        %{"player_id" => player_id, "angle" => angle},
        socket
      ) do
    notify_game_server_player_rotate(socket, player_id, angle)

    {:noreply, socket}
  end

  def handle_in(
        "player_jump",
        %{"player_id" => player_id},
        socket
      ) do
    notify_game_server_player_jump(socket, player_id)

    {:noreply, socket}
  end

  def handle_info({:game_updated, %State.Game{} = game_state}, socket) do
    push(socket, "game:updated", GameUpdate.from_state(game_state))

    {:noreply, socket}
  end

  defp init_socket(socket, game_id, %Player{} = user) do
    socket
    |> assign(:user, user)
    |> assign(:game_id, game_id)
  end

  defp game_server_pid() do
    SettleIt.Supervisor
    |> Supervisor.which_children()
    |> Enum.find_value(fn {process_name, pid, _, _} ->
      if process_name == SettleIt.GameServer.Registry, do: pid
    end)
  end

  defp notify_game_server(socket, msg) do
    GenServer.cast(
      game_server_pid(),
      {socket.assigns.game_id, msg}
    )
  end

  defp call_game_server(socket, msg) do
    GenServer.call(
      game_server_pid(),
      {socket.assigns.game_id, msg}
    )
  end

  defp join_game_server(socket) do
    call_game_server(
      socket,
      {:player_join, socket.assigns.user, self()}
    )
  end

  defp leave_game_server(socket) do
    notify_game_server(socket, {:player_leave, socket.assigns.user.id})
  end

  defp notify_game_server_start_game(socket) do
    notify_game_server(socket, :start_game)
  end

  defp notify_game_server_new_game(socket) do
    notify_game_server(socket, :restart_game)
  end

  defp notify_game_server_player_move(socket, player_id, coords) do
    notify_game_server(socket, {:player_move, player_id, coords})
  end

  defp notify_game_server_player_rotate(socket, player_id, angle) do
    notify_game_server(socket, {:player_rotate, player_id, angle})
  end

  defp notify_game_server_player_jump(socket, player_id) do
    notify_game_server(socket, {:player_jump, player_id})
  end
end
