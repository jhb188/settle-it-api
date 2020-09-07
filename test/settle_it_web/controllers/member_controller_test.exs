defmodule SettleItWeb.MemberControllerTest do
  use SettleItWeb.ConnCase

  alias SettleIt.Teams.Member

  @create_attrs %{}
  @update_attrs %{}
  @invalid_attrs %{player_id: Ecto.UUID.generate()}

  def fixture(:member) do
    SettleIt.TeamsTest.member_fixture(@create_attrs)
  end

  setup %{conn: conn} do
    {:ok, conn: put_req_header(conn, "accept", "application/json")}
  end

  describe "index" do
    test "lists all members", %{conn: conn} do
      conn = get(conn, Routes.member_path(conn, :index))
      assert json_response(conn, 200) == []
    end
  end

  describe "create member" do
    test "renders member when data is valid", %{conn: conn} do
      player = SettleIt.PlayersTest.player_fixture()
      team = SettleIt.TeamsTest.team_fixture()

      create_attrs = Map.merge(@create_attrs, %{player_id: player.id, team_id: team.id})

      conn = post(conn, Routes.member_path(conn, :create), create_attrs)
      assert %{"id" => id} = json_response(conn, 201)

      conn = get(conn, Routes.member_path(conn, :show, id))

      assert %{
               "id" => id
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, Routes.member_path(conn, :create), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update member" do
    setup [:create]

    test "renders member when data is valid", %{conn: conn, member: %Member{id: id} = member} do
      game = SettleIt.GamesTest.game_fixture()
      player = SettleIt.PlayersTest.player_fixture()
      update_attrs = Map.merge(@update_attrs, %{game_id: game.id, player_id: player.id})
      conn = put(conn, Routes.member_path(conn, :update, member), update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)

      conn = get(conn, Routes.member_path(conn, :show, id))

      assert %{
               "id" => id
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn, member: member} do
      conn = put(conn, Routes.member_path(conn, :update, member), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete member" do
    setup [:create]

    test "deletes chosen member", %{conn: conn, member: member} do
      conn = delete(conn, Routes.member_path(conn, :delete, member))
      assert response(conn, 204)

      assert_error_sent 404, fn ->
        get(conn, Routes.member_path(conn, :show, member))
      end
    end
  end

  defp create(_) do
    member = fixture(:member)
    {:ok, member: member}
  end
end
