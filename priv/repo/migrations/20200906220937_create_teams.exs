defmodule SettleIt.Repo.Migrations.CreateTeams do
  use Ecto.Migration

  def change do
    create table(:teams, primary_key: false) do
      add :id, :binary_id, primary_key: true
      add :cause, :string
      add :game_id, references(:games, on_delete: :nothing, type: :binary_id), null: false

      timestamps()
    end

    create index(:teams, [:game_id])
  end
end
