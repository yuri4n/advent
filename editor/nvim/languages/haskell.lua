return function()
  return {
    server = {
      cmd = { "haskell-language-server-wrapper", "--lsp" },
      filetypes = { "haskell", "lhaskell", "cabal" },
      settings = { haskell = { formattingProvider = "fourmolu" } },
    },
  }
end
