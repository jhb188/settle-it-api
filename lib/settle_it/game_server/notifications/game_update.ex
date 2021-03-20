defmodule SettleIt.GameServer.Notifications.GameUpdate do
  alias SettleIt.GameServer.State
  alias SettleIt.GameServer.Physics.Body

  def from_state(%State.Game{} = game_state) do
    %{
      # status: game_state.status,
      # players:
      #   game_state.players
      #   |> Map.values()
      #   |> Enum.map(&encode_player/1),
      bodies: Enum.map(game_state.bodies, &encode_body/1),
      last_updated: game_state.last_updated
    }
  end

  defp encode_player(player) do
    %{
      id: player.id,
      name: player.name
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
