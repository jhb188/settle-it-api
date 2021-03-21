defmodule SettleIt.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  def start(_type, _args) do
    # List all child processes to be supervised
    children = [
      {Cachex, name: :game_server_pids},
      # Start the GameServer supervisor and registry
      SettleIt.GameSupervisor,
      SettleIt.GameServer.Registry,
      # Start the endpoint when the application starts
      SettleItWeb.Endpoint
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: SettleIt.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  def config_change(changed, _new, removed) do
    SettleItWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
