defmodule Trigram.Native do
  @moduledoc false

  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :trigram,
    crate: "trigram_nif",
    base_url: "https://github.com/EnaiaInc/trigram/releases/download/v#{version}",
    force_build: System.get_env("TRIGRAM_BUILD") in ["1", "true"],
    version: version,
    nif_versions: ["2.17", "2.16", "2.15"],
    targets: [
      "aarch64-apple-darwin",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
      "x86_64-unknown-linux-gnu"
    ]

  @spec similarity(String.t(), String.t()) :: float()
  def similarity(_a, _b), do: :erlang.nif_error(:nif_not_loaded)

  @spec similarity_batch([{String.t(), String.t()}]) :: [float()]
  def similarity_batch(_pairs), do: :erlang.nif_error(:nif_not_loaded)

  @spec best_match(String.t(), [String.t()]) ::
          {:ok, {non_neg_integer(), float()}} | {:error, :empty_list}
  def best_match(_needle, _haystacks), do: :erlang.nif_error(:nif_not_loaded)

  @spec score_all(String.t(), [String.t()], float()) :: [{non_neg_integer(), float()}]
  def score_all(_needle, _haystacks, _min_threshold), do: :erlang.nif_error(:nif_not_loaded)
end
