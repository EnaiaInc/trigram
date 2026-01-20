# Exclude pg_trgm parity tests by default unless explicitly enabled
exclude =
  if System.get_env("TEST_PG_TRGM_PARITY") == "true" do
    []
  else
    [:pg_trgm_parity]
  end

ExUnit.start(exclude: exclude)
