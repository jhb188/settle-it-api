defmodule SettleIt.GameServer.State.Game do
  defstruct id: nil,
            status: :pending,
            players: %{},
            teams: %{},
            bodies: %{},
            topic: "",
            last_updated: nil

  def get_subscribed_processes(%__MODULE__{players: players}) do
    players
    |> Map.values()
    |> Enum.map(& &1.pid)
    |> Enum.filter(fn pid -> not is_nil(pid) end)
  end

  def won?(%__MODULE__{} = state) do
    length(
      Enum.filter(state.teams, fn {_team_id, team} ->
        length(
          Enum.filter(team.player_ids, fn player_id ->
            Map.get(state.bodies, player_id).hp > 0
          end)
        ) >
          0
      end)
    ) == 1
  end
end
