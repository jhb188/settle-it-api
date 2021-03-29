defmodule SettleItWeb.GameChannel do
  use SettleItWeb, :channel

  alias SettleIt.GameServer.State

  def join("game:" <> game_id, %{"player_id" => id, "topic" => topic}, socket) do
    with player <- %State.Player{name: "", id: id},
         socket <- init_socket(socket, game_id, player),
         :ok <- start_game_server(socket, topic) do
      {:ok, :ok, socket}
    else
      nil -> {:error, :not_found}
      error -> error
    end
  end

  def join("game:" <> game_id, %{"player_id" => id}, socket) do
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

  def handle_in("create_team", %{"player_id" => player_id, "name" => name}, socket) do
    notify_game_server_create_team(socket, player_id, name)

    {:noreply, socket}
  end

  def handle_in("delete_team", %{"team_id" => team_id}, socket) do
    notify_game_server_delete_team(socket, team_id)

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

  def handle_in("player_join_team", %{"player_id" => player_id, "team_id" => team_id}, socket) do
    notify_game_server_player_join_team(socket, player_id, team_id)

    {:noreply, socket}
  end

  def handle_in("player_update_name", %{"player_id" => player_id, "name" => name}, socket) do
    notify_game_server_player_update_name(socket, player_id, name)

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

  def handle_info({:bodies_updated, bodies_update}, socket) do
    push(socket, "bodies:updated", bodies_update)

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

  defp start_game_server(socket, topic) do
    notify_game_server(
      socket,
      {:player_start_lobby, socket.assigns.player, self(), topic}
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

  defp notify_game_server_player_shoot(socket, player_id, position, velocity) do
    notify_game_server(socket, {:player_shoot, player_id, position, velocity})
  end

  defp notify_game_server_create_team(socket, player_id, team_name) do
    notify_game_server(socket, {:create_team, player_id, team_name})
  end

  defp notify_game_server_player_join_team(socket, player_id, team_id) do
    notify_game_server(socket, {:player_join_team, player_id, team_id})
  end

  defp notify_game_server_delete_team(socket, team_id) do
    notify_game_server(socket, {:delete_team, team_id})
  end

  defp notify_game_server_player_update_name(socket, player_id, name) do
    notify_game_server(socket, {:player_update_name, player_id, name})
  end
end
