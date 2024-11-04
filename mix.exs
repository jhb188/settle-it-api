defmodule SettleIt.MixProject do
  use Mix.Project

  def project do
    [
      app: :settle_it,
      version: "0.1.0",
      elixir: "~> 1.14",
      elixirc_paths: elixirc_paths(Mix.env()),
      compilers: Mix.compilers(),
      start_permanent: Mix.env() == :prod,
      aliases: aliases(),
      deps: deps()
    ]
  end

  # Configuration for the OTP application.
  def application do
    [
      mod: {SettleIt.Application, []},
      extra_applications: [:cachex, :logger, :runtime_tools]
    ]
  end

  # Specifies which paths to compile per environment.
  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  # Specifies your project dependencies.
  defp deps do
    [
      {:phoenix, "~> 1.7"},
      {:phoenix_pubsub, "~> 2.1"},
      {:jason, "~> 1.0"},
      {:plug_cowboy, "~> 2.7"},
      {:gen_stage, "~> 1.0"},
      {:rustler, "~> 0.35.0"},
      {:math, "~> 0.6.0"},
      {:uuid, "~> 1.1"},
      {:cachex, "~> 4.0"}
    ]
  end

  defp aliases do
    [
      test: ["test"]
    ]
  end
end
