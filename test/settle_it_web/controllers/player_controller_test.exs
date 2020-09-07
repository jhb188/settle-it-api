defmodule SettleItWeb.PlayerControllerTest do
  use SettleItWeb.ConnCase

  alias SettleIt.Players
  alias SettleIt.Players.Player

  @create_attrs %{
    name: "some name"
  }
  @update_attrs %{
    name: "some updated name"
  }
  @invalid_attrs %{name: nil}

  def fixture(:player) do
    {:ok, player} = Players.create(@create_attrs)
    player
  end

  setup %{conn: conn} do
    {:ok, conn: put_req_header(conn, "accept", "application/json")}
  end

  describe "index" do
    test "lists all players", %{conn: conn} do
      conn = get(conn, Routes.player_path(conn, :index))
      assert json_response(conn, 200) == []
    end
  end

  describe "create player" do
    test "renders player when data is valid", %{conn: conn} do
      conn = post(conn, Routes.player_path(conn, :create), @create_attrs)
      assert %{"id" => id} = json_response(conn, 201)

      conn = get(conn, Routes.player_path(conn, :show, id))

      assert %{
               "id" => id,
               "name" => "some name"
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, Routes.player_path(conn, :create), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update player" do
    setup [:create]

    test "renders player when data is valid", %{conn: conn, player: %Player{id: id} = player} do
      conn = put(conn, Routes.player_path(conn, :update, player), @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)

      conn = get(conn, Routes.player_path(conn, :show, id))

      assert %{
               "id" => id,
               "name" => "some updated name"
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn, player: player} do
      conn = put(conn, Routes.player_path(conn, :update, player), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete player" do
    setup [:create]

    test "deletes chosen player", %{conn: conn, player: player} do
      conn = delete(conn, Routes.player_path(conn, :delete, player))
      assert response(conn, 204)

      assert_error_sent 404, fn ->
        get(conn, Routes.player_path(conn, :show, player))
      end
    end
  end

  defp create(_) do
    player = fixture(:player)
    {:ok, player: player}
  end
end
