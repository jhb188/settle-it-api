defmodule SettleIt.GameServer.Engine.MessageTest do
  use ExUnit.Case, async: true
  alias SettleIt.GameServer.Engine.Message

  describe "decode/1" do
    test "returns :game_won when input message indicates game win" do
      assert Message.decode("\"game_won\"\n") == :game_won
    end

    test "parses and decodes valid JSON messages into game bodies" do
      msg =
        "{\"1\":{\"id\":\"1\",\"class\":\"player\",\"translation\":[0,1,2],\"rotation\":[0,0,0],\"linvel\":[0,0,0],\"angvel\":[0,0,0],\"dimensions\":[1,1,1],\"mass\":10,\"hp\":100}}"

      assert {bodies, _extra} = Message.decode(msg)

      assert Map.has_key?(bodies, "1")
      assert bodies["1"].class == "player"
    end
  end
end
