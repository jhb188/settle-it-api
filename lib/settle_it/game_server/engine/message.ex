defmodule SettleIt.GameServer.Engine.Message do
  require Logger
  alias SettleIt.GameServer.State

  @spec decode(msg :: String.t()) :: :game_won | {bodies :: [%State.Body{}], extra :: String.t()}
  def decode(msg) do
    msg
    |> String.split("\n")
    |> decode_candidates()
  end

  defp decode_candidates(candidates, accum \\ %{})

  defp decode_candidates([], accum) do
    {accum, ""}
  end

  defp decode_candidates([skip_str | rest], accum) when skip_str in ["", "{}"] do
    decode_candidates(rest, accum)
  end

  defp decode_candidates([current], accum) do
    case decode_candidate(current) do
      {:ok, :game_won} -> :game_won
      {:ok, bodies} -> {Map.merge(accum, bodies), ""}
      :error -> {accum, current}
    end
  end

  defp decode_candidates([current | rest], accum) do
    case decode_candidate(current) do
      {:ok, :game_won} -> :game_won
      {:ok, bodies} -> decode_candidates(rest, Map.merge(accum, bodies))
      :error -> decode_candidates(rest, accum)
    end
  end

  defp decode_candidate(candidate) do
    try do
      {:ok, candidate |> Jason.decode!() |> decode_bodies()}
    rescue
      Jason.DecodeError ->
        Logger.error("Failed to decode message: #{inspect(candidate)}")
        :error
    end
  end

  defp decode_bodies("game_won") do
    :game_won
  end

  defp decode_bodies(raw_bodies) do
    raw_bodies
    |> Enum.map(fn {k, raw_body} -> {k, decode_body(raw_body)} end)
    |> Map.new()
  end

  defp decode_body(raw_body) do
    %State.Body{
      id: raw_body["id"],
      team_id: raw_body["team_id"],
      owner_id: raw_body["owner_id"],
      class: raw_body["class"],
      translation: decode_vec3(raw_body, "translation"),
      rotation: decode_vec3(raw_body, "rotation"),
      linvel: decode_vec3(raw_body, "linvel"),
      angvel: decode_vec3(raw_body, "angvel"),
      dimensions: decode_vec3(raw_body, "dimensions"),
      mass: raw_body["mass"],
      hp: raw_body["hp"]
    }
  end

  defp decode_vec3(raw_body, key) do
    [x, y, z] = raw_body[key]
    {x, y, z}
  end
end
