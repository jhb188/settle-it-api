defmodule SettleIt.GameServerSupervisor do
  use Supervisor

  alias SettleIt.GameServer
  alias SettleIt.GameServer.NotificationsDispatcher
  alias SettleIt.GameServer.StateProducer

  def start_link(game_id) do
    Supervisor.start_link(__MODULE__, game_id,
      name: String.to_atom("game_server_supervisor_" <> game_id)
    )
  end

  @impl true
  def init(game_id) do
    children = [
      {GameServer, game_id},
      {StateProducer, game_id},
      {NotificationsDispatcher, game_id}
    ]

    Supervisor.init(children, strategy: :rest_for_one)
  end
end
