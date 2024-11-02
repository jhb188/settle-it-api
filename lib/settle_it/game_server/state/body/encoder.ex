defmodule SettleIt.GameServer.State.Body.Encoder do
  defimpl Jason.Encoder, for: Tuple do
    def encode(data, options) when is_tuple(data) do
      data
      |> Tuple.to_list()
      |> Jason.Encoder.List.encode(options)
    end
  end
end
