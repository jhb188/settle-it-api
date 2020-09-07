defmodule SettleIt.TeamsTest do
  use SettleIt.DataCase

  alias SettleIt.Teams

  describe "teams" do
    alias SettleIt.Teams.Team

    @valid_attrs %{cause: "some cause"}
    @update_attrs %{cause: "some updated cause"}
    @invalid_attrs %{cause: nil}

    def team_fixture(attrs \\ %{}) do
      game = SettleIt.GamesTest.game_fixture()

      {:ok, team} =
        attrs
        |> Map.put(:game_id, game.id)
        |> Enum.into(@valid_attrs)
        |> Teams.create()

      team
    end

    test "list/0 returns all teams" do
      team = team_fixture()
      assert Teams.list() == [team]
    end

    test "get!/1 returns the team with given id" do
      team = team_fixture()
      assert Teams.get!(team.id) == team
    end

    test "create/1 with valid data creates a team" do
      game = SettleIt.GamesTest.game_fixture()
      valid_attrs = Map.put(@valid_attrs, :game_id, game.id)
      assert {:ok, %Team{} = team} = Teams.create(valid_attrs)
      assert team.cause == "some cause"
    end

    test "create/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Teams.create(@invalid_attrs)
    end

    test "update/2 with valid data updates the team" do
      team = team_fixture()
      assert {:ok, %Team{} = team} = Teams.update(team, @update_attrs)
      assert team.cause == "some updated cause"
    end

    test "update/2 with invalid data returns error changeset" do
      team = team_fixture()
      assert {:error, %Ecto.Changeset{}} = Teams.update(team, @invalid_attrs)
      assert team == Teams.get!(team.id)
    end

    test "delete/1 deletes the team" do
      team = team_fixture()
      assert {:ok, %Team{}} = Teams.delete(team)
      assert_raise Ecto.NoResultsError, fn -> Teams.get!(team.id) end
    end
  end

  describe "team_members" do
    alias SettleIt.Teams.Member

    @valid_attrs %{}
    @update_attrs %{}
    @invalid_attrs %{player_id: Ecto.UUID.generate()}

    def member_fixture(attrs \\ %{}) do
      team = SettleIt.TeamsTest.team_fixture()
      player = SettleIt.PlayersTest.player_fixture()

      {:ok, member} =
        attrs
        |> Map.merge(%{team_id: team.id, player_id: player.id})
        |> Enum.into(@valid_attrs)
        |> Teams.create_member()

      member
    end

    test "list_members/0 returns all team_members" do
      member = member_fixture()
      assert Teams.list_members() == [member]
    end

    test "get_member!/1 returns the member with given id" do
      member = member_fixture()
      assert Teams.get_member!(member.id) == member
    end

    test "create_member/1 with valid data creates a member" do
      team = SettleIt.TeamsTest.team_fixture()
      player = SettleIt.PlayersTest.player_fixture()
      valid_attrs = Map.merge(@valid_attrs, %{team_id: team.id, player_id: player.id})
      assert {:ok, %Member{} = member} = Teams.create_member(valid_attrs)
    end

    test "create_member/1 with invalid data returns error changeset" do
      assert {:error, %Ecto.Changeset{}} = Teams.create_member(@invalid_attrs)
    end

    test "update_member/2 with valid data updates the member" do
      member = member_fixture()
      assert {:ok, %Member{} = member} = Teams.update_member(member, @update_attrs)
    end

    test "update_member/2 with invalid data returns error changeset" do
      member = member_fixture()
      assert {:error, %Ecto.Changeset{}} = Teams.update_member(member, @invalid_attrs)
      assert member == Teams.get_member!(member.id)
    end

    test "delete_member/1 deletes the member" do
      member = member_fixture()
      assert {:ok, %Member{}} = Teams.delete_member(member)
      assert_raise Ecto.NoResultsError, fn -> Teams.get_member!(member.id) end
    end
  end
end
