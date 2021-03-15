defmodule SettleIt.GameServer.Physics do
  use Rustler, otp_app: :settle_it, crate: "physics"

  alias SettleIt.GameServer.Physics.Body

  @distance_from_center 50.0
  @height 1.0
  @player_mass 100.0

  @spec init_world() :: [Body]
  def init_world(), do: error()

  @spec step(bodies :: [Body], dt :: float()) :: [Body]
  def step(_bodies, _dt), do: error()

  @spec apply_jump(body :: Body) :: Body
  def apply_jump(_body), do: error()

  @spec add_player(bodies :: [Body], new_player_id :: String.t()) :: [Body]
  def add_player(bodies, new_player_id) do
    {players, nonplayers} = split_players_and_nonplayers(bodies)

    player_ids =
      (Enum.map(players, & &1.id) ++ [new_player_id])
      |> Enum.uniq()

    nonplayers ++ get_spaced_player_bodies(player_ids)
  end

  @spec remove_player(bodies :: [Body], new_player_id :: String.t()) :: [Body]
  def remove_player(bodies, player_id_to_remove) do
    {players, nonplayers} = split_players_and_nonplayers(bodies)

    player_ids = players |> Enum.map(& &1.id) |> Enum.reject(&(&1 == player_id_to_remove))

    nonplayers ++ get_spaced_player_bodies(player_ids)
  end

  defp get_spaced_player_bodies([]), do: []

  defp get_spaced_player_bodies(player_ids) do
    num_players = length(player_ids)
    circumference = 2 * :math.pi()
    angle_size = circumference / num_players

    player_ids
    |> Enum.with_index()
    |> Enum.map(fn {player_id, i} ->
      current_angle = angle_size * i
      x = @distance_from_center * :math.cos(current_angle)
      y = @distance_from_center * :math.sin(current_angle)
      z = @height / 2

      # angle 0 corresponds to an orientation resting on the x axis, looking in the direction of
      # the y axis, with the z axis being directly up. rotating (2pi / 4) counterclockwise makes the
      # orientation face the origin
      rotation = Math.rad2deg(circumference - current_angle - circumference / 4)

      %Body{
        id: player_id,
        translation: {x, y, z},
        rotation: {0.0, 0.0, rotation},
        mass: @player_mass,
        class: :player
      }
    end)
  end

  defp split_players_and_nonplayers(bodies) do
    Enum.split_with(bodies, fn body -> body.class == :player end)
  end

  defp error(), do: :erlang.nif_error(:nif_not_loaded)
end
