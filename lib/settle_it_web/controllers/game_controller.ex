defmodule SettleItWeb.GameController do
  use SettleItWeb, :controller

  alias SettleIt.Games
  alias SettleIt.Games.Game

  action_fallback SettleItWeb.FallbackController

  def index(conn, _params) do
    games = Games.list()
    render(conn, "index.json", games: games)
  end

  def create(conn, game_params) do
    with {:ok, %Game{} = game} <- Games.create(game_params) do
      conn
      |> put_status(:created)
      |> put_resp_header("location", Routes.game_path(conn, :show, game))
      |> render("show.json", game: game)
    end
  end

  def show(conn, %{"id" => id}) do
    game = Games.get!(id)
    render(conn, "show.json", game: game)
  end

  def update(conn, %{"id" => id} = game_params) do
    game = Games.get!(id)

    with {:ok, %Game{} = game} <- Games.update(game, game_params) do
      render(conn, "show.json", game: game)
    end
  end

  def delete(conn, %{"id" => id}) do
    game = Games.get!(id)

    with {:ok, %Game{}} <- Games.delete(game) do
      send_resp(conn, :no_content, "")
    end
  end
end
