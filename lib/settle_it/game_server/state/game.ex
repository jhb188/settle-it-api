defmodule SettleIt.GameServer.State.Game do
  defstruct players: %{},
            status: :pending,
            bodies: [],
            last_updated: nil

  def pending?(%__MODULE__{status: :pending}), do: true
  def pending?(_), do: false

  def empty?(%__MODULE__{players: players}) when players == %{}, do: true
  def empty?(_), do: false

  def get_subscribed_processes(%__MODULE__{players: players}) do
    players
    |> Map.values()
    |> Enum.map(& &1.pid)
    |> Enum.filter(fn pid -> not is_nil(pid) end)
  end
end
