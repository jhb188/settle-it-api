defmodule SettleIt.Teams.Team do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id
  schema "teams" do
    field :cause, :string
    field :game_id, :binary_id

    timestamps()
  end

  @doc false
  def changeset(team, attrs) do
    team
    |> cast(attrs, [:cause, :game_id])
    |> validate_required([:cause, :game_id])
    |> foreign_key_constraint(:game_id)
  end
end
