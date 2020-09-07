defmodule SettleItWeb.TeamControllerTest do
  use SettleItWeb.ConnCase

  alias SettleIt.Teams.Team

  @create_attrs %{
    cause: "some cause"
  }
  @update_attrs %{
    cause: "some updated cause"
  }
  @invalid_attrs %{cause: nil, game_id: nil}

  def fixture(:team) do
    SettleIt.TeamsTest.team_fixture(@create_attrs)
  end

  setup %{conn: conn} do
    {:ok, conn: put_req_header(conn, "accept", "application/json")}
  end

  describe "index" do
    test "lists all teams", %{conn: conn} do
      conn = get(conn, Routes.team_path(conn, :index))
      assert json_response(conn, 200) == []
    end
  end

  describe "create team" do
    test "renders team when data is valid", %{conn: conn} do
      game = SettleIt.GamesTest.game_fixture()
      create_attrs = Map.put(@create_attrs, :game_id, game.id)
      conn = post(conn, Routes.team_path(conn, :create), create_attrs)
      assert %{"id" => id} = json_response(conn, 201)

      conn = get(conn, Routes.team_path(conn, :show, id))

      assert %{
               "id" => id,
               "cause" => "some cause"
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, Routes.team_path(conn, :create), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update team" do
    setup [:create]

    test "renders team when data is valid", %{conn: conn, team: %Team{id: id} = team} do
      conn = put(conn, Routes.team_path(conn, :update, team), @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)

      conn = get(conn, Routes.team_path(conn, :show, id))

      assert %{
               "id" => id,
               "cause" => "some updated cause"
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn, team: team} do
      conn = put(conn, Routes.team_path(conn, :update, team), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete team" do
    setup [:create]

    test "deletes chosen team", %{conn: conn, team: team} do
      conn = delete(conn, Routes.team_path(conn, :delete, team))
      assert response(conn, 204)

      assert_error_sent 404, fn ->
        get(conn, Routes.team_path(conn, :show, team))
      end
    end
  end

  defp create(_) do
    team = fixture(:team)
    {:ok, team: team}
  end
end
