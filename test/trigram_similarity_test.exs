defmodule Trigram.SimilarityTest do
  use ExUnit.Case, async: true

  alias Trigram
  alias Trigram.Support.SimilarityCases

  test "native similarity matches Elixir implementation" do
    mismatches =
      SimilarityCases.pairs()
      |> Enum.map(fn {left, right} ->
        elixir_score = Trigram.Elixir.similarity(left, right)
        native_score = Trigram.similarity(left, right)
        {left, right, elixir_score, native_score}
      end)
      |> Enum.reject(fn {_left, _right, elixir_score, native_score} ->
        elixir_score == native_score
      end)

    assert mismatches == [],
           "mismatches: " <>
             inspect(mismatches, limit: :infinity, printable_limit: :infinity)
  end

  test "similarity_batch matches per-pair similarity" do
    pairs =
      SimilarityCases.pairs()
      |> Enum.take(10)

    expected =
      Enum.map(pairs, fn {left, right} ->
        Trigram.similarity(left, right)
      end)

    assert Trigram.similarity_batch(pairs) == expected
  end

  test "best_match returns lowest index on ties" do
    needle = "hello world"
    haystacks = ["hello world", "hello world", "hullo world"]

    assert {:ok, {0, 1.0}} = Trigram.best_match(needle, haystacks)
  end

  test "score_all filters and sorts by score then index" do
    needle = "hello"
    haystacks = ["hello", "hallo", "help", "world"]

    results = Trigram.score_all(needle, haystacks, 0.3)

    assert results == Enum.sort_by(results, fn {idx, score} -> {-score, idx} end)
    assert Enum.all?(results, fn {_idx, score} -> score >= 0.3 end)
  end
end
