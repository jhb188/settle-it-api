defmodule SettleIt.GameServer.State.Team do
  defstruct id: nil, owner_id: nil, cause: "", player_ids: MapSet.new(), color: ""
end
