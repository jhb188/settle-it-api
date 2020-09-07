defmodule SettleIt.PlayersTest do
  use SettleIt.DataCase

  alias SettleIt.Players

  describe "players" do
    alias SettleIt.Players.Player

    @valid_attrs %{name: "some name"}
    @update_attrs %{name: "some updated name"}
    @invalid_attrs %{name: nil}

    def player_fixture(attrs \\ %{}) do
      {:ok, player} =
        attrs
        |> Enum.into(@valid_attrs)
        |> Players.create()

      player
    end

    test "list/0 returns all players" do
      player = player_fixture()
      assert Players.list() == [player]
    end

    test "get!/1 returns the player with given id" do
      player = player_fixture()
      assert Players.get!(player.id) == player
    end

    test "create/1 with valid data creates a player" do
      assert {:ok, %Player{} = player} = Players.create(@valid_attrs)
      assert player.name == "some name"
    end

    test "create/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Players.create(@invalid_attrs)
    end

    test "update/2 with valid data updates the player" do
      player = player_fixture()
      assert {:ok, %Player{} = player} = Players.update(player, @update_attrs)
      assert player.name == "some updated name"
    end

    test "update/2 with invalid data returns error changeset" do
      player = player_fixture()
      assert {:error, %Ecto.Changeset{}} = Players.update(player, @invalid_attrs)
      assert player == Players.get!(player.id)
    end

    test "delete/1 deletes the player" do
      player = player_fixture()
      assert {:ok, %Player{}} = Players.delete(player)
      assert_raise Ecto.NoResultsError, fn -> Players.get!(player.id) end
    end
  end
end
