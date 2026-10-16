-- Assertions must run after VimEnter, just as in an interactive session.
vim.api.nvim_create_autocmd("VimEnter", {
  once = true,
  callback = function()
    vim.defer_fn(function()
      local ok, message = xpcall(function()
        dofile(vim.env.AOC_TEST_ASSERTIONS)
        assert(vim.v.errmsg == "", vim.v.errmsg)
      end, debug.traceback)
      -- Let fixture servers finish shutdown before their temporary log/cache
      -- directories are removed by the Python parent process.
      local function clients()
        return vim.lsp.get_clients({ _uninitialized = true })
      end
      vim.lsp.stop_client(clients())
      if not vim.wait(1000, function() return #clients() == 0 end, 10) then
        vim.lsp.stop_client(clients(), true)
        vim.wait(1000, function() return #clients() == 0 end, 10)
      end
      if ok then
        print("PASS: " .. vim.env.AOC_TEST_NAME)
        vim.cmd("qa!")
      else
        io.stderr:write(tostring(message) .. "\n")
        vim.cmd("cquit 1")
      end
    end, 20)
  end,
})
