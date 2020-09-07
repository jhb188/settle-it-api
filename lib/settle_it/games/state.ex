defmodule SettleIt.Games.State do
  use EctoEnum.Postgres, type: :game_state, enums: [:pending, :started, :finished]
end
