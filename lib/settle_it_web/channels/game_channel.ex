defmodule SettleItWeb.GameChannel do
  use SettleItWeb, :channel

  alias SettleIt.GameServer.State

  def join("game:" <> game_id, id, socket) do
    with player <- %State.Player{name: "", id: id},
         socket <- init_socket(socket, game_id, player),
         :ok <- join_game_server(socket) do
      {:ok, :ok, socket}
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

  def handle_in(
        "player_shoot",
        %{
          "player_id" => player_id,
          "position" => %{"x" => position_x, "y" => position_y, "z" => position_z},
          "linvel" => %{"x" => linvel_x, "y" => linvel_y, "z" => linvel_z}
        },
        socket
      ) do
    position = %{x: position_x, y: position_y, z: position_z}
    linvel = %{x: linvel_x, y: linvel_y, z: linvel_z}

    notify_game_server_player_shoot(socket, player_id, position, linvel)

    {:noreply, socket}
  end

  def handle_info({:game_updated, game_update}, socket) do
    push(socket, "game:updated", game_update)

    {:noreply, socket}
  end

  defp init_socket(socket, game_id, %State.Player{} = player) do
    socket
    |> assign(:player, player)
    |> assign(:game_id, game_id)
  end

  defp game_server_registry_pid() do
    case Cachex.fetch(
           :game_server_pids,
           :game_server_registry,
           fn _ ->
             SettleIt.Supervisor
             |> Supervisor.which_children()
             |> Enum.find_value(fn {process_name, pid, _, _} ->
               if process_name == SettleIt.GameServer.Registry, do: {:commit, pid}
             end)
           end,
           ttl: 30000
         ) do
      {:ok, pid} -> pid
      {:commit, pid} -> pid
    end
  end

  defp notify_game_server(socket, msg) do
    GenServer.cast(
      game_server_registry_pid(),
      {socket.assigns.game_id, msg}
    )
  end

  defp join_game_server(socket) do
    notify_game_server(
      socket,
      {:player_join, socket.assigns.player, self()}
    )
  end

  defp leave_game_server(socket) do
    notify_game_server(socket, {:player_leave, socket.assigns.player.id})
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

  def notify_game_server_player_shoot(socket, player_id, position, velocity) do
    notify_game_server(socket, {:player_shoot, player_id, position, velocity})
  end
end
