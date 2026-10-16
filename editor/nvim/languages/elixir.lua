return function()
  return {
    aliases = { "elixirls", "elixir-ls", "elixir_ls" },
    server = {
      cmd = { "elixir-ls" },
      filetypes = { "elixir", "eelixir", "heex" },
    },
  }
end
