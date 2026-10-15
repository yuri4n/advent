return function(context)
  return {
    server = {
      cmd = { "rust-analyzer" },
      filetypes = { "rust" },
      settings = {
        ["rust-analyzer"] = { linkedProjects = { context.project_root .. "/Cargo.toml" } },
      },
    },
  }
end
