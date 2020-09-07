defmodule SettleIt.Repo do
  use Ecto.Repo,
    otp_app: :settle_it,
    adapter: Ecto.Adapters.Postgres
end
