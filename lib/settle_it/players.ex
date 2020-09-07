defmodule SettleIt.Players do
  @moduledoc """
  The Players context.
  """

  import Ecto.Query, warn: false
  alias SettleIt.Repo

  alias SettleIt.Players.Player

  @doc """
  Returns the list of players.

  ## Examples

      iex> list()
      [%Player{}, ...]

  """
  def list do
    Repo.all(Player)
  end

  @doc """
  Gets a single player.

  Raises `Ecto.NoResultsError` if the Player does not exist.

  ## Examples

      iex> get!(123)
      %Player{}

      iex> get!(456)
      ** (Ecto.NoResultsError)

  """
  def get!(id), do: Repo.get!(Player, id)

  @doc """
  Creates a player.

  ## Examples

      iex> create(%{field: value})
      {:ok, %Player{}}

      iex> create(%{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def create(attrs \\ %{}) do
    %Player{}
    |> Player.changeset(attrs)
    |> Repo.insert()
  end

  @doc """
  Updates a player.

  ## Examples

      iex> update(player, %{field: new_value})
      {:ok, %Player{}}

      iex> update(player, %{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def update(%Player{} = player, attrs) do
    player
    |> Player.changeset(attrs)
    |> Repo.update()
  end

  @doc """
  Deletes a player.

  ## Examples

      iex> delete(player)
      {:ok, %Player{}}

      iex> delete(player)
      {:error, %Ecto.Changeset{}}

  """
  def delete(%Player{} = player) do
    Repo.delete(player)
  end
end
