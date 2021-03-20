defmodule SettleItWeb.Router do
  use SettleItWeb, :router

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/api", SettleItWeb do
    pipe_through :api
  end
end
