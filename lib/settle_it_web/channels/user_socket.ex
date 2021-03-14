defmodule SettleItWeb.UserSocket do
  use Phoenix.Socket

  channel "game:*", SettleItWeb.GameChannel

  def connect(_params, socket, _connect_info) do
    {:ok, socket}
  end

  def id(_socket), do: nil
end
