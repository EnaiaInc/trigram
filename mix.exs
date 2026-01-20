defmodule Trigram.MixProject do
  use Mix.Project

  @version "0.6.0"
  @source_url "https://github.com/EnaiaInc/trigram"

  def project do
    [
      app: :trigram,
      version: @version,
      elixir: "~> 1.19",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: description(),
      package: package(),
      docs: docs()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.37.1", optional: true},
      {:rustler_precompiled, "~> 0.8.4"},
      {:postgrex, "~> 0.22.0", only: :test},
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.39.3", only: :dev, runtime: false},
      {:quokka, "~> 2.11", only: [:dev, :test], runtime: false}
    ]
  end

  defp description do
    "PostgreSQL pg_trgm-compatible trigram similarity for Elixir with a Rust NIF."
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{
        "Changelog" => "#{@source_url}/blob/main/CHANGELOG.md",
        "GitHub" => @source_url
      },
      files: [
        "lib",
        "native/trigram_nif/.cargo",
        "native/trigram_nif/src",
        "native/trigram_nif/Cargo*",
        "native/trigram_nif/Cross.toml",
        "checksum-*.exs",
        "mix.exs",
        "README.md",
        "CHANGELOG.md",
        "LICENSE"
      ]
    ]
  end

  defp docs do
    [
      main: "Trigram",
      source_url: @source_url,
      source_ref: "v#{@version}"
    ]
  end
end
