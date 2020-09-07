defmodule SettleItWeb.PlayerController do
  use SettleItWeb, :controller

  alias SettleIt.Players
  alias SettleIt.Players.Player

  action_fallback SettleItWeb.FallbackController

  def index(conn, _params) do
    players = Players.list()
    render(conn, "index.json", players: players)
  end

  def create(conn, player_params) do
    with {:ok, %Player{} = player} <- Players.create(player_params) do
      conn
      |> put_status(:created)
      |> put_resp_header("location", Routes.player_path(conn, :show, player))
      |> render("show.json", player: player)
    end
  end

  def show(conn, %{"id" => id}) do
    player = Players.get!(id)
    render(conn, "show.json", player: player)
  end

  def update(conn, %{"id" => id} = player_params) do
    player = Players.get!(id)

    with {:ok, %Player{} = player} <- Players.update(player, player_params) do
      render(conn, "show.json", player: player)
    end
  end

  def delete(conn, %{"id" => id}) do
    player = Players.get!(id)

    with {:ok, %Player{}} <- Players.delete(player) do
      send_resp(conn, :no_content, "")
    end
  end
end
