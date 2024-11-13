defmodule SettleIt.GameServer.Physics do
  @skip_compilation? is_nil(System.find_executable("cargo"))
  @mode if Mix.env() == :prod || System.get_env("RUST_ENV") == "prod", do: :release, else: :debug

  use Rustler,
    otp_app: :settle_it,
    crate: :physics,
    mode: @mode,
    skip_compilation?: @skip_compilation?
end
