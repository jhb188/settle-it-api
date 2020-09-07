defmodule SettleIt.Repo.Migrations.CreateGames do
  use Ecto.Migration

  def change do
    SettleIt.Games.State.create_type()

    create table(:games, primary_key: false) do
      add :id, :binary_id, primary_key: true
      add :creator_id, references(:players, on_delete: :nothing, type: :binary_id), null: false
      add :state, SettleIt.Games.State.type()

      timestamps()
    end

    create index(:games, [:creator_id])
  end
end
