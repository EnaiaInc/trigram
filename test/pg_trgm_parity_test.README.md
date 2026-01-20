# PostgreSQL pg_trgm Parity Test

This test verifies that the Trigram library's implementation exactly matches PostgreSQL's `pg_trgm` extension behavior.

## Why is this test separate?

The parity test is optional and excluded by default because it:

1. **Requires external infrastructure**: A PostgreSQL database with the `pg_trgm` extension installed
2. **Has additional dependencies**: The `postgrex` Elixir library (marked as optional)
3. **Is not needed for normal development**: The main test suite verifies that the Rust NIF matches the Elixir implementation, which is already verified to match pg_trgm

## Running the test

### Prerequisites

1. A running PostgreSQL instance
2. The `pg_trgm` extension installed:

```sql
CREATE EXTENSION IF NOT EXISTS pg_trgm;
```

### Running with default database

```bash
TEST_PG_TRGM_PARITY=true mix test test/pg_trgm_parity_test.exs
```

This will attempt to connect to: `postgres://postgres:postgres@localhost/postgres`

### Running with custom database

```bash
TEST_PG_TRGM_PARITY=true DATABASE_URL=postgres://user:pass@host:port/dbname mix test test/pg_trgm_parity_test.exs
```

## What this test verifies

The test compares the similarity scores calculated by:

- PostgreSQL's `pg_trgm` extension (via SQL query)
- The Trigram library's Elixir implementation

It uses a comprehensive set of test cases including:

- UTF-8 characters (café, naïve, über)
- Non-Latin scripts (Cyrillic, Chinese, Greek)
- Punctuation and special characters
- Word separators (spaces, underscores, hyphens)

## Architecture

The test is designed to run independently without requiring:

- Ecto.Repo
- Phoenix DataCase
- Application-specific test infrastructure

It directly uses `Postgrex` to query the database and compares results against the library's implementation.

## Patterns learned from reference implementations

This test follows patterns found in:

1. **ex_aws** - Optional dependencies marked in mix.exs
2. **enaia/enaia** - Environment-based test exclusion in test_helper.exs
3. **Common Elixir test patterns** - Using `@moduletag` with ExUnit.configure exclude lists

The key pattern is:

```elixir
# In test_helper.exs
exclude = if System.get_env("TEST_FLAG") == "true", do: [], else: [:tag_name]
ExUnit.start(exclude: exclude)

# In test file
@moduletag :tag_name
```

This allows tests to be completely skipped unless explicitly enabled via environment variable.
