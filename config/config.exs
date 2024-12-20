# This file is responsible for configuring your application
# and its dependencies with the aid of the Mix.Config module.
#
# This configuration file is loaded before any dependency and
# is restricted to this project.

# General application configuration
import Config

config :settle_it,
  generators: [binary_id: true]

# Configures the endpoint
config :settle_it, SettleItWeb.Endpoint,
  url: [host: "localhost"],
  secret_key_base: "e2CBrQA9Rcq0xpimSL+P5CodaFOA+YZlQtnY0Yf5099+9Nwt9QVtKSerfC0G07aU",
  render_errors: [view: SettleItWeb.ErrorView, accepts: ~w(json)],
  pubsub_server: SettleIt.PubSub

# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason

# Import environment specific config. This must remain at the bottom
# of this file so it overrides the configuration defined above.
import_config "#{Mix.env()}.exs"
