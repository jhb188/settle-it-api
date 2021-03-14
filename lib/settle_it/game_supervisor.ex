defmodule SettleIt.GameSupervisor do
  use DynamicSupervisor
  alias SettleIt.GameServer

  def start_link(_) do
    DynamicSupervisor.start_link(__MODULE__, %{}, name: __MODULE__)
  end

  def init(_arg) do
    DynamicSupervisor.init(strategy: :one_for_one)
  end

  def start_game_server(game_id) do
    attrs = %{
      id: GameServer,
      start: {GameServer, :start_link, [game_id]}
    }

    DynamicSupervisor.start_child(__MODULE__, attrs)
  end
end
