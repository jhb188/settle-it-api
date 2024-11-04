defmodule SettleIt.GameServer.Physics do
  @mode if Mix.env() == :prod || System.get_env("RUST_ENV") == "prod", do: :release, else: :debug

  use Rustler, otp_app: :settle_it, crate: :physics, path: "native/physics", mode: @mode
end
