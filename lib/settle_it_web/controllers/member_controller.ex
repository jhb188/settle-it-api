defmodule SettleItWeb.MemberController do
  use SettleItWeb, :controller

  alias SettleIt.Teams
  alias SettleIt.Teams.Member

  action_fallback SettleItWeb.FallbackController

  def index(conn, _params) do
    team_members = Teams.list_members()
    render(conn, "index.json", team_members: team_members)
  end

  def create(conn, member_params) do
    with {:ok, %Member{} = member} <- Teams.create_member(member_params) do
      conn
      |> put_status(:created)
      |> put_resp_header("location", Routes.member_path(conn, :show, member))
      |> render("show.json", member: member)
    end
  end

  def show(conn, %{"id" => id}) do
    member = Teams.get_member!(id)
    render(conn, "show.json", member: member)
  end

  def update(conn, %{"id" => id} = member_params) do
    member = Teams.get_member!(id)

    with {:ok, %Member{} = member} <- Teams.update_member(member, member_params) do
      render(conn, "show.json", member: member)
    end
  end

  def delete(conn, %{"id" => id}) do
    member = Teams.get_member!(id)

    with {:ok, %Member{}} <- Teams.delete_member(member) do
      send_resp(conn, :no_content, "")
    end
  end
end
