defmodule SettleIt.GameServer do
  @moduledoc """
  Runs the game
  """
  use GenStage

  def start_link(game_id) do
    GenStage.start_link(__MODULE__, game_id, name: String.to_atom(game_id))
  end

  @impl true
  def init(_game_id) do
    {:producer, []}
  end

  @impl true
  def handle_cast(message, state) do
    {:noreply, [message], state}
  end

  @impl true
  def handle_demand(_, state), do: {:noreply, [], state}
end
