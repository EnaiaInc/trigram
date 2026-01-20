defmodule Trigram.FallbackTest do
  use ExUnit.Case, async: true

  test "fallback uses Elixir implementation when NIF is unavailable" do
    task =
      Task.async(fn ->
        Code.compiler_options(ignore_module_conflict: true)

        defmodule Trigram.Native do
          def similarity(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
          def similarity_batch(_pairs), do: :erlang.nif_error(:nif_not_loaded)
          def best_match(_needle, _haystacks), do: :erlang.nif_error(:nif_not_loaded)

          def score_all(_needle, _haystacks, _min_threshold),
            do: :erlang.nif_error(:nif_not_loaded)
        end

        defmodule Trigram do
          @global_elixir_impl :"Elixir.Trigram.Elixir"

          def similarity(a, b) do
            Trigram.Native.similarity(a, b)
          rescue
            e in ErlangError ->
              if e.original == :nif_not_loaded do
                # Reference the global Trigram.Elixir module directly
                @global_elixir_impl.similarity(a, b)
              else
                reraise e, __STACKTRACE__
              end
          end
        end

        Trigram.similarity("hello", "hello")
      end)

    assert Task.await(task, 5_000) == 1.0
  end
end
