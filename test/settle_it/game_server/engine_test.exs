defmodule SettleIt.GameServer.EngineTest do
  use ExUnit.Case, async: true
  alias SettleIt.GameServer.Engine
  alias SettleIt.GameServer.State

  describe "init/1" do
    test "initializes a game state with the given ID" do
      game_id = "game_123"
      state = Engine.init(game_id)

      assert %State.Game{id: ^game_id} = state
      assert is_integer(state.last_updated)
    end
  end

  describe "create_team/3" do
    test "creates a new team when the cause is unique" do
      state = %State.Game{teams: %{}}
      owner_id = "player_1"
      player = %State.Player{id: owner_id, name: "John"}
      state = Engine.add_player(state, player, self())
      cause = "Save the Forest"

      updated_state = Engine.create_team(state, owner_id, cause)

      assert Enum.any?(updated_state.teams, fn {_id, team} -> team.cause == cause end)
    end

    test "does not create a team if the cause already exists" do
      existing_team = %State.Team{cause: "Save the Forest"}
      state = %State.Game{teams: %{"team_1" => existing_team}}

      updated_state = Engine.create_team(state, "player_2", "Save the Forest")

      assert updated_state == state
    end
  end

  describe "add_player/3" do
    test "adds a new player to the game state" do
      state = %State.Game{players: %{}}
      player = %State.Player{id: "player_1", name: "John"}
      pid = self()

      updated_state = Engine.add_player(state, player, pid)

      assert Map.has_key?(updated_state.players, "player_1")
      assert updated_state.players["player_1"].pid == pid
    end
  end
end
