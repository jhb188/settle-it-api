defmodule SettleIt.GameServer.State.Game do
  defstruct id: nil,
            status: :pending,
            players: %{},
            teams: %{},
            bodies: %{},
            world_bodies: %{},
            topic: "",
            last_updated: nil,
            physics_port: nil,
            physics_data: ""

  def get_subscribed_processes(%__MODULE__{players: players}) do
    players
    |> Map.values()
    |> Enum.map(& &1.pid)
    |> Enum.filter(fn pid -> not is_nil(pid) end)
  end
end
