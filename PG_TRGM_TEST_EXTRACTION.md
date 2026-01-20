# PostgreSQL pg_trgm Comparison Test - Extraction and Rework

## Summary

Successfully extracted the pg_trgm comparison test from PR 9743 (commit `cf8716f89`) in the `~/enaia/enaia` repository and reworked it to run as an optional, standalone test in the `/Users/neil/enaia/trigram` project without requiring Ecto.Repo or Phoenix DataCase dependencies.

## What Was Extracted

### Original Test (from PR 9743)

**Location**: `~/enaia/enaia/test/enaia/text/trigram_similarity_test.exs`

```elixir
defmodule Enaia.Text.TrigramSimilarityTest do
  use Enaia.DataCase, async: true

  alias Enaia.Text.TrigramSimilarity
  alias Enaia.Text.TrigramSimilarity.Cases

  test "matches pg_trgm similarity for diverse inputs" do
    Enum.each(Cases.pairs(), fn {left, right} ->
      pg_score = pg_similarity(left, right)
      ours = TrigramSimilarity.similarity(left, right)
      assert pg_score == ours
    end)
  end

  defp pg_similarity(left, right) do
    result = Repo.query!("SELECT similarity($1::text, $2::text)", [left, right])
    [[score]] = result.rows
    score
  end
end
```

**Test Cases**: 25 diverse input pairs including UTF-8 characters, non-Latin scripts (Cyrillic, Chinese, Greek), punctuation, and various word separators.

## How It Was Made Independent

### 1. Removed Ecto/DataCase Dependencies

**Original dependencies**:

- `use Enaia.DataCase` - Phoenix test case with Ecto sandbox
- `Repo.query!` - Ecto repository for database queries

**New approach**:

- `use ExUnit.Case, async: false` - Standard ExUnit test case
- Direct `Postgrex` connection and queries
- Custom database setup/teardown in `setup_all` callback

### 2. Made It Conditional with Environment Flag

**Environment Flag**: `TEST_PG_TRGM_PARITY`

**Implementation Pattern**:

```elixir
# In test_helper.exs
exclude =
  if System.get_env("TEST_PG_TRGM_PARITY") == "true" do
    []
  else
    [:pg_trgm_parity]
  end

ExUnit.start(exclude: exclude)
```

```elixir
# In test file
@moduletag :pg_trgm_parity

setup_all do
  if enabled?() do
    # Setup database connection
  else
    :ok  # Skip setup if not enabled
  end
end

defp enabled?() do
  System.get_env("TEST_PG_TRGM_PARITY") == "true"
end
```

### 3. Added Direct Database Management

**Database Connection**:

- Uses `Postgrex.start_link/1` directly
- Configurable via `DATABASE_URL` environment variable
- Default: `postgres://postgres:postgres@localhost/postgres`
- Verifies `pg_trgm` extension is installed before running tests

**Connection Lifecycle**:

- Established in `setup_all` callback
- Closed automatically with `on_exit` callback
- Test fails fast with clear message if extension is missing

### 4. Added Optional Dependency

**In mix.exs**:

```elixir
{:postgrex, "~> 0.19", only: :test, optional: true}
```

This makes Postgrex available for testing but doesn't require it for normal library usage.

## Files Created/Modified

### Created Files

1. **/Users/neil/enaia/trigram/test/pg_trgm_parity_test.exs**
   - Standalone test with embedded documentation
   - Direct Postgrex integration
   - Environment flag controlled execution
   - Clear error messages for missing dependencies

2. **/Users/neil/enaia/trigram/test/pg_trgm_parity_test.README.md**
   - Comprehensive documentation
   - Setup instructions
   - Usage examples
   - Architecture explanation
   - Patterns learned from reference implementations

3. **/Users/neil/enaia/trigram/PG_TRGM_TEST_EXTRACTION.md** (this file)
   - Complete summary of extraction and rework process

### Modified Files

1. **/Users/neil/enaia/trigram/test/test_helper.exs**
   - Added conditional exclusion logic based on `TEST_PG_TRGM_PARITY` environment variable
   - Follows pattern from `~/enaia/enaia/test/test_helper.exs`

2. **/Users/neil/enaia/trigram/mix.exs**
   - Added `{:postgrex, "~> 0.19", only: :test, optional: true}` dependency

3. **/Users/neil/enaia/trigram/lib/trigram/native.ex**
   - Fixed variable scoping issue (moved `version` assignment before `use RustlerPrecompiled`)
   - Added `nif_versions` configuration

## Environment Flag Control

### When Flag is NOT Set (Default Behavior)

```bash
mix test
```

- Test is completely skipped
- No database connection attempted
- No Postgrex dependency required
- Exit code: 0 (success)

### When Flag IS Set

```bash
TEST_PG_TRGM_PARITY=true mix test test/pg_trgm_parity_test.exs
```

- Test runs and connects to PostgreSQL
- Verifies pg_trgm extension is installed
- Compares Trigram library output with PostgreSQL pg_trgm
- Exit code: 0 if all similarities match, 1 if any mismatch

### Custom Database URL

```bash
TEST_PG_TRGM_PARITY=true DATABASE_URL=postgres://user:pass@host/db mix test
```

## Patterns Learned from Reference Implementations

### From ~/enaia/enaia (test/test_helper.exs)

```elixir
exclude = [:integration_test, :pending]

exclude =
  if System.get_env("GITHUB_ACTIONS") == "true" do
    [:local_only | exclude]
  else
    [:ci_only | exclude]
  end

ExUnit.configure(exclude: exclude)
```

**Pattern**: Build exclusion list conditionally based on environment variables, then pass to `ExUnit.configure/1` or `ExUnit.start/1`.

### From ~/xuku/ex_aws\* Projects

```elixir
# Optional dependencies in mix.exs
{:ex_aws, "~> 2.0", optional: true}
```

**Pattern**: Mark dependencies as `optional: true` when they're not required for core functionality but enable additional features (like testing against real services).

### From Multiple Repositories

```elixir
@moduletag :integration
@moduletag :skip

if System.get_env("RUN_INTEGRATION"), do: @tag(run: true)
```

**Pattern**: Use `@moduletag` to mark entire test modules, allowing fine-grained control over which tests run based on tags and environment configuration.

## Key Architecture Decisions

### 1. No Application Context Required

- Test runs without starting any OTP applications
- No need for application configuration files
- Pure unit test that happens to use a database for verification

### 2. Fail Fast with Clear Messages

- Checks for database connectivity immediately
- Verifies pg_trgm extension exists before running
- Provides actionable error messages

### 3. Self-Contained Documentation

- Test file includes usage instructions in @moduledoc
- Separate README for detailed architecture explanation
- Examples for all common usage patterns

### 4. Zero Impact on Default Test Suite

- Completely skipped by default
- No additional dependencies loaded
- No performance impact on regular test runs

## Verification Commands

### Run only the parity test (when enabled):

```bash
TEST_PG_TRGM_PARITY=true mix test test/pg_trgm_parity_test.exs
```

### Run all tests (parity test will be skipped):

```bash
mix test
```

### Verify test is excluded by default:

```bash
mix test --trace | grep -i "pg_trgm\|excluded"
```

## Test Case Coverage

The test uses 25 diverse string pairs from `Trigram.SimilarityCases`:

- **Diacritics**: café/cafe, naïve/naive, résumé/resume, façade/facade
- **German**: über/uber, straße/strasse
- **Portuguese**: São/Sao
- **Swedish**: ångström/angstrom
- **French**: fiancé/fiance
- **Cyrillic**: привет/privet
- **Chinese**: 東京 variants
- **Greek**: Ελλάδα/Ellada
- **Turkish**: İstanbul/istanbul
- **Punctuation**: LLC/L.L.C., $1,000.00/$1000.00
- **Separators**: foo_bar/foo bar, hello-world/hello world, co-op/coop
- **Whitespace**: space tabs/space tabs

## Success Criteria

- Test compiles without Ecto.Repo or DataCase dependencies
- Test is skipped by default when running full test suite
- Test runs when `TEST_PG_TRGM_PARITY=true` is set
- Test can connect to custom database via `DATABASE_URL`
- Test provides clear error if pg_trgm extension is missing
- Test verifies all 25 test cases match PostgreSQL pg_trgm behavior
- Pattern can be reused for other optional integration tests

## Next Steps (For Future Work)

1. Actually run the test against a PostgreSQL database to verify it works
2. Consider adding more edge cases to the test suite
3. Potentially add this pattern to CI/CD with a dedicated test database
4. Document this pattern for other optional integration tests
