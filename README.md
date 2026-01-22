# Trigram

PostgreSQL `pg_trgm`-compatible trigram similarity for Elixir, with a fast Rust NIF and a pure Elixir fallback.

## Installation

Add the dependency to your `mix.exs`:

```elixir
def deps do
  [
    {:trigram, "~> 0.6.0"}
  ]
end
```

## Usage

```elixir
Trigram.similarity("hello", "hallo")
Trigram.best_match("hello", ["world", "hallo", "help"])
Trigram.score_all("hello", ["world", "hallo", "help"], 0.3)
```

## Precompiled NIFs

This library uses `rustler_precompiled` and will download precompiled NIFs on compile. To force
local compilation instead, set:

```bash
export TRIGRAM_BUILD=1
```

## Development

See [RELEASE.md](RELEASE.md) for instructions on creating releases and managing precompiled binaries.

## License

MIT

## Changelog

See [CHANGELOG.md](CHANGELOG.md).
