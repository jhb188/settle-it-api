defmodule SettleIt.Games do
  @moduledoc """
  The Games context.
  """

  import Ecto.Query, warn: false
  alias SettleIt.Repo

  alias SettleIt.Games.Game

  @doc """
  Returns the list of games.

  ## Examples

      iex> list()
      [%Game{}, ...]

  """
  def list do
    Repo.all(Game)
  end

  @doc """
  Gets a single game.

  Raises `Ecto.NoResultsError` if the Game does not exist.

  ## Examples

      iex> get!(123)
      %Game{}

      iex> get!(456)
      ** (Ecto.NoResultsError)

  """
  def get!(id), do: Repo.get!(Game, id)

  @doc """
  Creates a game.

  ## Examples

      iex> create(%{field: value})
      {:ok, %Game{}}

      iex> create(%{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def create(attrs \\ %{}) do
    %Game{}
    |> Game.changeset(attrs)
    |> Repo.insert()
  end

  @doc """
  Updates a game.

  ## Examples

      iex> update(game, %{field: new_value})
      {:ok, %Game{}}

      iex> update(game, %{field: bad_value})
      {:error, %Ecto.Changeset{}}

  """
  def update(%Game{} = game, attrs) do
    game
    |> Game.changeset(attrs)
    |> Repo.update()
  end

  @doc """
  Deletes a game.

  ## Examples

      iex> delete(game)
      {:ok, %Game{}}

      iex> delete(game)
      {:error, %Ecto.Changeset{}}

  """
  def delete(%Game{} = game) do
    Repo.delete(game)
  end
end
