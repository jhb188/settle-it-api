defmodule SettleIt.MixProject do
  use Mix.Project

  def project do
    [
      app: :settle_it,
      version: "0.1.0",
      elixir: "~> 1.5",
      elixirc_paths: elixirc_paths(Mix.env()),
      compilers: compilers() ++ Mix.compilers(),
      rustler_crates: rustler_crates(),
      start_permanent: Mix.env() == :prod,
      aliases: aliases(),
      deps: deps()
    ]
  end

  # Configuration for the OTP application.
  #
  # Type `mix help compile.app` for more information.
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
  #
  # Type `mix help deps` for examples and options.
  defp deps do
    [
      {:phoenix, "~> 1.4.11"},
      {:phoenix_pubsub, "~> 1.1"},
      {:jason, "~> 1.0"},
      {:plug_cowboy, "~> 2.0"},
      {:gen_stage, "~> 1.0"},
      {:rustler, "~> 0.21.1"},
      {:math, "~> 0.6.0"},
      {:uuid, "~> 1.1"},
      {:cachex, "~> 3.3"}
    ]
  end

  # attempt to hack around rebuilding NIF
  defp compilers() do
    case System.find_executable("cargo") do
      nil -> [:phoenix]
      _ -> [:rustler, :phoenix]
    end
  end

  defp aliases do
    [
      test: ["test"]
    ]
  end

  defp rustler_crates do
    [
      physics: [
        path: "native/physics",
        mode: rustc_mode(Mix.env(), System.get_env("RUST_ENV"))
      ]
    ]
  end

  defp rustc_mode(:prod, _), do: :release
  defp rustc_mode(_, "prod"), do: :release
  defp rustc_mode(_, _), do: :debug
end
