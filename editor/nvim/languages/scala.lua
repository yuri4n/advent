return function(context)
  return {
    server = {
      cmd = { "metals" },
      filetypes = { "scala", "sbt" },
      -- Use standard LSP features without requiring nvim-metals extensions.
      init_options = { statusBarProvider = "off" },
    },
    prepare = function(start_server)
      -- BSP launchers contain machine-specific paths. Refresh them inside the
      -- chosen toolchain without blocking editing while Scala CLI runs.
      local scala_cli = vim.env.SCALA_CLI or "scala-cli"
      if vim.fn.executable(scala_cli) ~= 1 then
        vim.notify("scala-cli is missing from PATH; enter nix develop .#scala before starting the editor.", vim.log.levels.WARN)
        return
      end
      vim.notify("Preparing Scala CLI build metadata for Metals…")
      vim.system({ scala_cli, "setup-ide", ".", "--server=false" }, {
        cwd = context.project_root,
        text = true,
        timeout = 120000,
      }, vim.schedule_wrap(function(result)
        if result.code ~= 0 then
          local details = vim.trim(result.stderr or "")
          if details == "" then
            details = vim.trim(result.stdout or "")
          end
          vim.notify(
            "Scala CLI setup-ide failed; run `scala-cli setup-ide . --server=false` in "
              .. context.project_root .. ":\n" .. details,
            vim.log.levels.ERROR
          )
          return
        end
        start_server()
      end))
    end,
  }
end
