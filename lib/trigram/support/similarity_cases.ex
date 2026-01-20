defmodule Trigram.Support.SimilarityCases do
  @moduledoc false

  @pairs [
    {"café", "cafe"},
    {"naïve", "naive"},
    {"über", "uber"},
    {"São", "Sao"},
    {"ångström", "angstrom"},
    {"fiancé", "fiance"},
    {"привет", "privet"},
    {"東京", "东 京"},
    {"東京", "東京"},
    {"foo_bar", "foo bar"},
    {"foo_bar", "foobar"},
    {"hello-world", "hello world"},
    {"hello—world", "hello world"},
    {"$1,000.00", "$1000.00"},
    {"Apt #4B", "Apt 4B"},
    {"LLC", "L.L.C."},
    {"co-op", "coop"},
    {"123-456", "123456"},
    {"mid–range", "mid range"},
    {"space   tabs", "space tabs"},
    {"résumé", "resume"},
    {"façade", "facade"},
    {"İstanbul", "istanbul"},
    {"straße", "strasse"},
    {"Ελλάδα", "Ellada"}
  ]

  def pairs, do: @pairs
end
