return function()
  return {
    aliases = { "gopls" },
    server = {
      cmd = { "gopls" },
      filetypes = { "go", "gomod", "gowork", "gotmpl" },
    },
  }
end
