defmodule SettleItWeb.MemberView do
  use SettleItWeb, :view
  alias SettleItWeb.MemberView

  def render("index.json", %{team_members: team_members}) do
    render_many(team_members, MemberView, "member.json")
  end

  def render("show.json", %{member: member}) do
    render_one(member, MemberView, "member.json")
  end

  def render("member.json", %{member: member}) do
    %{id: member.id}
  end
end
