defmodule SettleIt.Teams.Member do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id
  schema "team_members" do
    field :team_id, :binary_id
    field :player_id, :binary_id

    timestamps()
  end

  @doc false
  def changeset(member, attrs) do
    member
    |> cast(attrs, [:team_id, :player_id])
    |> validate_required([:team_id, :player_id])
    |> foreign_key_constraint(:team_id)
    |> foreign_key_constraint(:player_id)
  end
end
