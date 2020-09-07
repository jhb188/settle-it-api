defmodule SettleItWeb.TeamController do
  use SettleItWeb, :controller

  alias SettleIt.Teams
  alias SettleIt.Teams.Team

  action_fallback SettleItWeb.FallbackController

  def index(conn, _params) do
    teams = Teams.list()
    render(conn, "index.json", teams: teams)
  end

  def create(conn, team_params) do
    with {:ok, %Team{} = team} <- Teams.create(team_params) do
      conn
      |> put_status(:created)
      |> put_resp_header("location", Routes.team_path(conn, :show, team))
      |> render("show.json", team: team)
    end
  end

  @spec show(Plug.Conn.t(), map) :: Plug.Conn.t()
  def show(conn, %{"id" => id}) do
    team = Teams.get!(id)
    render(conn, "show.json", team: team)
  end

  def update(conn, %{"id" => id} = team_params) do
    team = Teams.get!(id)

    with {:ok, %Team{} = team} <- Teams.update(team, team_params) do
      render(conn, "show.json", team: team)
    end
  end

  def delete(conn, %{"id" => id}) do
    team = Teams.get!(id)

    with {:ok, %Team{}} <- Teams.delete(team) do
      send_resp(conn, :no_content, "")
    end
  end
end
