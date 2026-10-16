return function(context)
  return {
    aliases = { "metals" },
    status = {
      starting = "Starting Metals…",
      attached = "Metals attached. Build import and diagnostics may still be running.",
      reusing = "Using your existing Metals configuration.",
      failed = "Metals stopped. Run :checkhealth vim.lsp for its server log.",
    },
    server = {
      cmd = { "metals" },
      filetypes = { "scala", "sbt" },
      -- Use standard LSP features without requiring nvim-metals extensions.
      init_options = {
        statusBarProvider = "off",
        bspStatusBarProvider = "show-message",
        isHttpEnabled = false,
        doctorVisibilityProvider = true,
      },
      on_init = function(client)
        -- This client has no Doctor panel. Report it hidden so background
        -- checks do not prompt to start the optional HTTP Doctor interface.
        client:notify("metals/doctorVisibilityDidChange", { visible = false })
      end,
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
      local started = vim.uv.hrtime()
      -- A Java-wrapper installation can otherwise record its non-executable
      -- JAR as argv[0], leaving Metals unable to launch the BSP server.
      vim.system({ scala_cli, "--initial-launcher-path", vim.fn.exepath(scala_cli), "setup-ide", ".", "--server=false" }, {
        cwd = context.project_root,
        text = true,
        timeout = 120000,
      }, vim.schedule_wrap(function(result)
        if result.code ~= 0 then
          local details = vim.trim(result.stderr or "")
          if details == "" then
            details = vim.trim(result.stdout or "")
          end
          if details == "" then
            details = "exit code " .. tostring(result.code)
          end
          vim.notify(
            "Scala CLI setup-ide failed. From the repository root, run "
              .. "`bash scripts/languages/scala.sh editor-setup`:\n" .. details:sub(-4000),
            vim.log.levels.ERROR
          )
          return
        end
        vim.notify(string.format("Scala CLI build metadata ready (%.1fs).", (vim.uv.hrtime() - started) / 1e9))
        start_server()
      end))
    end,
  }
end
