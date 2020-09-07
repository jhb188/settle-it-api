defmodule SettleItWeb.Router do
  use SettleItWeb, :router

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/api", SettleItWeb do
    pipe_through :api

    resources "/players", PlayerController, except: [:new, :edit]
    resources "/games", GameController, except: [:new, :edit]
    resources "/teams", TeamController, except: [:new, :edit]
    resources "/team_members", MemberController, except: [:new, :edit]
  end
end
