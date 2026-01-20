defmodule Trigram do
  @moduledoc """
  PostgreSQL pg_trgm-compatible trigram similarity for Elixir.

  This module uses a Rust NIF for performance, with a pure Elixir
  fallback when the NIF is unavailable.
  """

  alias Trigram.Elixir, as: ElixirImpl
  alias Trigram.Native

  @doc """
  Calculate trigram similarity between two strings.

  Returns a float between 0.0 and 1.0, where 1.0 means exact match.
  """
  @spec similarity(String.t(), String.t()) :: float()
  def similarity(a, b) do
    with_native(fn -> Native.similarity(a, b) end, fn -> ElixirImpl.similarity(a, b) end)
  end

  @doc """
  Calculate trigram similarity for multiple pairs.
  """
  @spec similarity_batch([{String.t(), String.t()}]) :: [float()]
  def similarity_batch(pairs) do
    with_native(
      fn -> Native.similarity_batch(pairs) end,
      fn -> ElixirImpl.similarity_batch(pairs) end
    )
  end

  @doc """
  Find the best match for a needle in a list of haystacks.
  """
  @spec best_match(String.t(), [String.t()]) ::
          {:ok, {non_neg_integer(), float()}} | {:error, :empty_list}
  def best_match(needle, haystacks) do
    with_native(fn -> Native.best_match(needle, haystacks) end, fn ->
      ElixirImpl.best_match(needle, haystacks)
    end)
  end

  @doc """
  Score all haystacks against a needle and return results above threshold.
  """
  @spec score_all(String.t(), [String.t()], float()) :: [{non_neg_integer(), float()}]
  def score_all(needle, haystacks, min_threshold) do
    with_native(
      fn -> Native.score_all(needle, haystacks, min_threshold) end,
      fn -> ElixirImpl.score_all(needle, haystacks, min_threshold) end
    )
  end

  defp with_native(native_fun, fallback_fun) do
    native_fun.()
  rescue
    e in ErlangError ->
      if e.original == :nif_not_loaded do
        fallback_fun.()
      else
        reraise e, __STACKTRACE__
      end
  end
end
