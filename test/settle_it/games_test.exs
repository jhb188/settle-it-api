defmodule SettleIt.GamesTest do
  use SettleIt.DataCase

  alias SettleIt.Games

  describe "games" do
    alias SettleIt.Games.Game

    @valid_attrs %{}
    @update_attrs %{}
    @invalid_attrs %{creator_id: Ecto.UUID.generate()}

    def game_fixture(attrs \\ %{}) do
      player = SettleIt.PlayersTest.player_fixture()
      attrs = Map.put(attrs, :creator_id, player.id)

      {:ok, game} =
        attrs
        |> Enum.into(@valid_attrs)
        |> Games.create()

      game
    end

    test "list/0 returns all games" do
      game = game_fixture()
      assert Games.list() == [game]
    end

    test "get!/1 returns the game with given id" do
      game = game_fixture()
      assert Games.get!(game.id) == game
    end

    test "create/1 with valid data creates a game" do
      player = SettleIt.PlayersTest.player_fixture()
      valid_attrs = Map.put(@valid_attrs, :creator_id, player.id)
      assert {:ok, %Game{} = game} = Games.create(valid_attrs)
    end

    test "create/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Games.create(@invalid_attrs)
    end

    test "update/2 with valid data updates the game" do
      game = game_fixture()
      assert {:ok, %Game{} = game} = Games.update(game, @update_attrs)
    end

    test "update/2 with invalid data returns error changeset" do
      game = game_fixture()
      assert {:error, %Ecto.Changeset{}} = Games.update(game, @invalid_attrs)
      assert game == Games.get!(game.id)
    end

    test "delete/1 deletes the game" do
      game = game_fixture()
      assert {:ok, %Game{}} = Games.delete(game)
      assert_raise Ecto.NoResultsError, fn -> Games.get!(game.id) end
    end
  end
end
