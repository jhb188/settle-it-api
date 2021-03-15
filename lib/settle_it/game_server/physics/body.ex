defmodule SettleIt.GameServer.Physics.Body do
  defstruct id: "",
            translation: {0.0, 0.0, 0.0},
            rotation: {0.0, 0.0, 0.0},
            linvel: {0.0, 0.0, 0.0},
            angvel: {0.0, 0.0, 0.0},
            mass: 0.0,
            class: nil
end
