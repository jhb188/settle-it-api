defmodule SettleIt.GameServer.Notifications.GameUpdate do
  alias SettleIt.GameServer.State
  alias SettleIt.GameServer.Physics.Body

  def from_state(%State.Game{} = game_state) do
    %{
      id: game_state.id,
      status: game_state.status,
      players: Enum.map(game_state.players, fn {_id, player} -> encode_player(player) end),
      teams: Enum.map(game_state.teams, fn {_id, team} -> encode_team(team) end),
      bodies: Enum.map(game_state.bodies, fn {_id, body} -> encode_body(body) end),
      topic: game_state.topic,
      last_updated: game_state.last_updated
    }
  end

  defp encode_player(player) do
    %{
      id: player.id,
      name: player.name
    }
  end

  defp encode_team(team) do
    %{
      id: team.id,
      owner_id: team.owner_id,
      cause: team.cause,
      player_ids: MapSet.to_list(team.player_ids),
      color: team.color
    }
  end

  defp encode_body(%Body{} = body) do
    %{
      id: body.id,
      translation: encode_vec3(body.translation),
      rotation: encode_rotation_vector(body.rotation),
      linvel: encode_vec3(body.linvel),
      angvel: encode_vec3(body.angvel),
      mass: body.mass,
      class: body.class,
      owner_id: body.owner_id,
      hp: body.hp
    }
  end

  defp encode_vec3({x, y, z}), do: %{x: x, y: y, z: z}

  defp encode_rotation_vector({x, y, z}), do: encode_vec3({x, y, z})
end
