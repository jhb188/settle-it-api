defmodule SettleItWeb.TeamView do
  use SettleItWeb, :view
  alias SettleItWeb.TeamView

  def render("index.json", %{teams: teams}) do
    render_many(teams, TeamView, "team.json")
  end

  def render("show.json", %{team: team}) do
    render_one(team, TeamView, "team.json")
  end

  def render("team.json", %{team: team}) do
    %{id: team.id, cause: team.cause}
  end
end
