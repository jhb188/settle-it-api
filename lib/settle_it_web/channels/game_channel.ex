defmodule SettleItWeb.GameChannel do
  require Logger
  use SettleItWeb, :channel

  alias SettleIt.GameServer.State

  def join("game:" <> game_id, %{"player_id" => id, "topic" => topic}, socket) do
    with player <- %State.Player{name: "", id: id},
         socket <- init_socket(socket, game_id, player),
         :ok <-
           notify_game_server(
             socket,
             {:player_start_lobby, socket.assigns.player, self(), topic}
           ) do
      {:ok, :ok, socket}
    else
      nil ->
        Logger.error("Join failed: game #{game_id} or player #{id} not found.")

      error ->
        Logger.error("Unexpected join error: #{inspect(error)}")
        error
    end
  end

  def join("game:" <> game_id, %{"player_id" => id}, socket) do
    with player <- %State.Player{name: "", id: id},
         socket <- init_socket(socket, game_id, player),
         :ok <-
           notify_game_server(
             socket,
             {:player_join, socket.assigns.player, self()}
           ) do
      {:ok, :ok, socket}
    else
      nil ->
        Logger.error("Join failed: game #{game_id} or player #{id} not found.")

      error ->
        Logger.error("Unexpected join error: #{inspect(error)}")
        error
    end
  end

  def terminate(_reason, socket) do
    notify_game_server(socket, {:player_leave, socket.assigns.player.id})

    {:noreply, socket}
  end

  def handle_in("create_team", %{"player_id" => player_id, "name" => team_name}, socket) do
    notify_game_server(socket, {:create_team, player_id, team_name})

    {:noreply, socket}
  end

  def handle_in("delete_team", %{"team_id" => team_id}, socket) do
    notify_game_server(socket, {:delete_team, team_id})

    {:noreply, socket}
  end

  def handle_in("start_game", _params, socket) do
    notify_game_server(socket, :start_game)

    {:noreply, socket}
  end

  def handle_in("restart_game", _game_id, socket) do
    notify_game_server(socket, :restart_game)

    {:noreply, socket}
  end

  def handle_in("player_join_team", %{"player_id" => player_id, "team_id" => team_id}, socket) do
    notify_game_server(socket, {:player_join_team, player_id, team_id})

    {:noreply, socket}
  end

  def handle_in("player_update_name", %{"player_id" => player_id, "name" => name}, socket) do
    notify_game_server(socket, {:player_update_name, player_id, name})

    {:noreply, socket}
  end

  def handle_in(
        "player_move",
        %{"player_id" => player_id, "x" => x, "y" => y},
        socket
      ) do
    notify_game_server(socket, {:player_move, player_id, %{x: x, y: y}})

    {:noreply, socket}
  end

  def handle_in(
        "player_rotate",
        %{"player_id" => player_id, "angle" => angle},
        socket
      ) do
    notify_game_server(socket, {:player_rotate, player_id, angle})

    {:noreply, socket}
  end

  def handle_in(
        "player_jump",
        %{"player_id" => player_id},
        socket
      ) do
    notify_game_server(socket, {:player_jump, player_id})

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

    notify_game_server(socket, {:player_shoot, player_id, position, linvel})

    {:noreply, socket}
  end

  def handle_in(event, data, _socket) do
    Logger.error("Received unexpected event #{event} with data #{inspect(data)}.")
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
end
