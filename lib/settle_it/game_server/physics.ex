defmodule SettleIt.GameServer.Physics do
  use Rustler, otp_app: :settle_it, crate: "physics"

  alias SettleIt.GameServer.Physics.Body

  @spec init_world() :: [Body]
  def init_world(), do: error()

  @spec step(bodies :: [Body], dt :: float()) :: [Body]
  def step(_bodies, _dt), do: error()

  @spec apply_jump(body :: Body) :: Body
  def apply_jump(_body), do: error()

  defp error(), do: :erlang.nif_error(:nif_not_loaded)
end
