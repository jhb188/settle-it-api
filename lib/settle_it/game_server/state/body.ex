defmodule SettleIt.GameServer.State.Body do
  defstruct id: nil,
            team_id: nil,
            owner_id: nil,
            class: nil,
            translation: {0.0, 0.0, 0.0},
            rotation: {0.0, 0.0, 0.0},
            linvel: {0.0, 0.0, 0.0},
            angvel: {0.0, 0.0, 0.0},
            dimensions: {0.0, 0.0, 0.0},
            mass: 0.0,
            hp: 0
end
