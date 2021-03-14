defmodule SettleIt.GameServer.Registry do
  use GenServer
  alias SettleIt.GameSupervisor

  def init(_state) do
    {:ok, %{}}
  end

  def start_link(_arg) do
    GenServer.start_link(__MODULE__, name: __MODULE__)
  end

  def handle_call({game_id, msg}, _from, registry) do
    route_to(registry, game_id, msg, &call/3)
  end

  def handle_cast({game_id, msg}, registry) do
    route_to(registry, game_id, msg, &cast/3)
  end

  defp route_to(registry, game_id, msg, call_fn) do
    case registry[game_id] do
      nil ->
        {:ok, pid} = get_game_server(game_id)
        call_fn.(Map.put(registry, game_id, pid), pid, msg)

      pid ->
        call_fn.(registry, pid, msg)
    end
  end

  defp call(registry, pid, msg) do
    {:reply, GenServer.call(pid, msg), registry}
  end

  def cast(registry, pid, msg) do
    GenServer.cast(pid, msg)
    {:noreply, registry}
  end

  defp get_game_server(game_id) do
    case GameSupervisor.start_game_server(game_id) do
      {:ok, pid} -> {:ok, pid}
      {:error, {:already_started, pid}} -> {:ok, pid}
      error -> error
    end
  end
end
