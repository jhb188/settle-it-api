defmodule SettleIt.Repo.Migrations.CreateTeamMembers do
  use Ecto.Migration

  def change do
    create table(:team_members, primary_key: false) do
      add :id, :binary_id, primary_key: true
      add :team_id, references(:teams, on_delete: :nothing, type: :binary_id), null: false
      add :player_id, references(:players, on_delete: :nothing, type: :binary_id), null: false

      timestamps()
    end

    create index(:team_members, [:team_id])
    create index(:team_members, [:player_id])
    create unique_index(:team_members, [:team_id, :player_id])
  end
end
