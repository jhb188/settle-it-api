defmodule SettleIt.GameServer.State.Body do
  defstruct id: nil,
            translation: {0.0, 0.0, 0.0},
            rotation: {0.0, 0.0, 0.0},
            linvel: {0.0, 0.0, 0.0},
            angvel: {0.0, 0.0, 0.0},
            mass: 0.0,
            class: nil,
            owner_id: nil,
            hp: 0
end
