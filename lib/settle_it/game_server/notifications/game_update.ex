defmodule SettleIt.GameServer.Notifications.GameUpdate do
  alias SettleIt.GameServer.State

  def from_state(%State.Game{} = game_state) do
    %{
      id: game_state.id,
      status: game_state.status,
      players: Enum.map(game_state.players, fn {_id, player} -> encode_player(player) end),
      teams: Enum.map(game_state.teams, fn {_id, team} -> encode_team(team) end),
      bodies: encode_bodies(game_state.bodies),
      topic: game_state.topic,
      last_updated: game_state.last_updated
    }
  end

  def bodies_update_from_state(%State.Game{} = game_state) do
    bodies =
      game_state.bodies
      |> Enum.reject(fn {_body_id, body} -> body.class == :obstacle end)
      |> encode_bodies()

    %{
      bodies: bodies,
      last_updated: game_state.last_updated
    }
  end

  defp encode_bodies(bodies) do
    Enum.map(bodies, fn {_id, body} -> encode_body(body) end)
  end

  defp encode_player(player) do
    %{
      id: player.id,
      name: player.name,
      team_id: player.team_id
    }
  end

  defp encode_team(team) do
    %{
      id: team.id,
      owner_id: team.owner_id,
      cause: team.cause,
      color: team.color
    }
  end

  defp encode_body(%State.Body{} = body) do
    %{
      id: body.id,
      tid: body.team_id,
      tra: encode_vec3(body.translation),
      rot: encode_rotation_vector(body.rotation),
      lv: encode_vec3(body.linvel),
      av: encode_vec3(body.angvel),
      d: encode_vec3(body.dimensions),
      m: body.mass,
      cl: body.class,
      hp: body.hp
    }
  end

  defp encode_vec3({x, y, z}), do: %{x: x, y: y, z: z}

  defp encode_rotation_vector({x, y, z}), do: encode_vec3({x, y, z})
end
