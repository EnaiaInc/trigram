# Used by "mix format"
[
  subdirectories: ["priv/*/migrations"],
  plugins: [Quokka],
  inputs: [
    "{mix,.formatter}.exs",
    "{config,test}/**/*.{ex,exs}",
    "lib/**/!(native).ex",
    "priv/*/seeds.exs"
  ],
  quokka: [
    autosort: [:map, :defstruct],
    exclude: [],
    only: [
      :blocks,
      :comment_directives,
      :configs,
      :defs,
      :deprecations,
      :module_directives,
      :pipes,
      :single_node
    ]
  ]
]
