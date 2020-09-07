defmodule SettleItWeb.GameControllerTest do
  use SettleItWeb.ConnCase

  alias SettleIt.Games
  alias SettleIt.Games.Game

  @create_attrs %{}
  @update_attrs %{}
  @invalid_attrs %{"creator_id" => Ecto.UUID.generate()}

  def fixture(:game) do
    player = SettleIt.PlayersTest.player_fixture()
    {:ok, game} = @create_attrs |> Map.put(:creator_id, player.id) |> Games.create()
    game
  end

  setup %{conn: conn} do
    {:ok, conn: put_req_header(conn, "accept", "application/json")}
  end

  describe "index" do
    test "lists all games", %{conn: conn} do
      conn = get(conn, Routes.game_path(conn, :index))
      assert json_response(conn, 200) == []
    end
  end

  describe "create game" do
    test "renders game when data is valid", %{conn: conn} do
      player = SettleIt.PlayersTest.player_fixture()
      create_attrs = Map.put(@create_attrs, :creator_id, player.id)
      conn = post(conn, Routes.game_path(conn, :create), create_attrs)
      assert %{"id" => id} = json_response(conn, 201)

      conn = get(conn, Routes.game_path(conn, :show, id))

      assert %{
               "id" => id
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn} do
      conn = post(conn, Routes.game_path(conn, :create), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "update game" do
    setup [:create]

    test "renders game when data is valid", %{conn: conn, game: %Game{id: id} = game} do
      conn = put(conn, Routes.game_path(conn, :update, game), @update_attrs)
      assert %{"id" => ^id} = json_response(conn, 200)

      conn = get(conn, Routes.game_path(conn, :show, id))

      assert %{
               "id" => id
             } = json_response(conn, 200)
    end

    test "renders errors when data is invalid", %{conn: conn, game: game} do
      conn = put(conn, Routes.game_path(conn, :update, game), @invalid_attrs)
      assert json_response(conn, 422)["errors"] != %{}
    end
  end

  describe "delete game" do
    setup [:create]

    test "deletes chosen game", %{conn: conn, game: game} do
      conn = delete(conn, Routes.game_path(conn, :delete, game))
      assert response(conn, 204)

      assert_error_sent 404, fn ->
        get(conn, Routes.game_path(conn, :show, game))
      end
    end
  end

  defp create(_) do
    game = fixture(:game)
    {:ok, game: game}
  end
end
