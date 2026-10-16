return function()
  return {
    aliases = { "hls", "haskell-language-server", "haskell-language-server-wrapper" },
    server = {
      cmd = { "haskell-language-server-wrapper", "--lsp" },
      filetypes = { "haskell", "lhaskell", "cabal" },
      settings = { haskell = { formattingProvider = "fourmolu" } },
    },
  }
end
