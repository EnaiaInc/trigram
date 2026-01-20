defmodule Trigram.PgTrgmParityTest do
  @moduledoc """
  Verifies Trigram implementation matches PostgreSQL pg_trgm extension behavior.

  This test requires a PostgreSQL database with pg_trgm extension enabled.
  It is skipped by default and only runs when TEST_PG_TRGM_PARITY=true.

  ## Running the test

  Set up a PostgreSQL database with pg_trgm extension:

      CREATE EXTENSION IF NOT EXISTS pg_trgm;

  Then run:

      TEST_PG_TRGM_PARITY=true mix test test/pg_trgm_parity_test.exs

  Or with custom database URL:

      TEST_PG_TRGM_PARITY=true DATABASE_URL=postgres://user:pass@localhost/dbname mix test
  """

  use ExUnit.Case, async: false

  alias Trigram
  alias Trigram.Support.SimilarityCases

  @moduletag :pg_trgm_parity

  setup_all do
    if enabled?() do
      case setup_database() do
        {:ok, conn} ->
          on_exit(fn -> GenServer.stop(conn) end)
          {:ok, %{conn: conn}}

        {:error, reason} ->
          flunk("Failed to connect to database: #{inspect(reason)}")
      end
    else
      :ok
    end
  end

  describe "pg_trgm parity" do
    test "matches pg_trgm similarity for diverse inputs", context do
      if enabled?() do
        conn = Map.fetch!(context, :conn)

        Enum.each(SimilarityCases.pairs(), fn {left, right} ->
          pg_score = pg_similarity(conn, left, right)
          our_score = Trigram.similarity(left, right)

          assert our_score == pg_score,
                 """
                 Mismatch for pair: #{inspect({left, right})}
                 PostgreSQL pg_trgm: #{pg_score}
                 Our implementation:  #{our_score}
                 """
        end)
      end
    end
  end

  # Private helpers

  defp enabled? do
    System.get_env("TEST_PG_TRGM_PARITY") == "true"
  end

  defp setup_database do
    db_url = System.get_env("DATABASE_URL") || "postgres://postgres:postgres@localhost/postgres"

    case Postgrex.start_link(connect_opts(db_url)) do
      {:ok, conn} ->
        case Postgrex.query(
               conn,
               "SELECT extname FROM pg_extension WHERE extname = 'pg_trgm'",
               []
             ) do
          {:ok, %{rows: [[_]]}} ->
            {:ok, conn}

          {:ok, %{rows: []}} ->
            {:error, "pg_trgm extension not installed. Run: CREATE EXTENSION pg_trgm;"}

          {:error, reason} ->
            {:error, reason}
        end

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp connect_opts(db_url) when is_binary(db_url) do
    uri = URI.parse(db_url)
    database = (uri.path || "") |> String.trim_leading("/")

    {username, password} =
      case uri.userinfo do
        nil ->
          {nil, nil}

        userinfo ->
          case String.split(userinfo, ":", parts: 2) do
            [user, pass] -> {user, pass}
            [user] -> {user, nil}
          end
      end

    [
      hostname: uri.host || "localhost",
      port: uri.port || 5432,
      database: if(database == "", do: "postgres", else: database),
      username: username,
      password: password
    ]
    |> Enum.reject(fn {_key, value} -> is_nil(value) end)
  end

  defp pg_similarity(conn, left, right) do
    {:ok, result} = Postgrex.query(conn, "SELECT similarity($1::text, $2::text)", [left, right])
    [[score]] = result.rows
    score
  end
end
