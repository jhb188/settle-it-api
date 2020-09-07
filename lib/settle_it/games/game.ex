defmodule SettleIt.Games.Game do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id
  schema "games" do
    field :creator_id, :binary_id
    field :state, SettleIt.Games.State, default: :pending

    timestamps()
  end

  @doc false
  def changeset(game, attrs) do
    game
    |> cast(attrs, [:creator_id, :state])
    |> foreign_key_constraint(:creator_id)
    |> validate_required([:creator_id])
  end
end
