return function(context)
  return {
    aliases = { "rust_analyzer", "rust-analyzer", "rustaceanvim" },
    server = {
      cmd = { "rust-analyzer" },
      filetypes = { "rust" },
      settings = {
        ["rust-analyzer"] = { linkedProjects = { context.project_root .. "/Cargo.toml" } },
      },
    },
  }
end
