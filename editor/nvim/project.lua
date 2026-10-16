-- Shared project overlay. Personal options, mappings, plugins, completion and
-- server configuration remain untouched. Language modules supply missing LSPs.
return function(options)
  if vim.fn.has("nvim-0.11") == 0 then
    vim.notify("The AoC overlay needs Neovim 0.11+; use the Nix shell or --user-config.", vim.log.levels.ERROR)
    return
  end
  local language = vim.env.AOC_EDITOR_LANGUAGE
  local root = vim.env.AOC_ROOT
  local project = vim.env.AOC_PROJECT_ROOT
  if not language or not root or not project then
    vim.notify("Start this configuration with: aoc edit YEAR DAY --lang LANGUAGE", vim.log.levels.WARN)
    return
  end
  if not language:match("^[a-z0-9_-]+$") then
    vim.notify("Invalid editor language: " .. language, vim.log.levels.ERROR)
    return
  end

  -- Add languages/<name>.lua alongside nix/languages/<name>.nix; no shared edit.
  -- Each module returns function(context), producing:
  --   { server = { cmd = { "server", "args" }, filetypes = { ... }, ... },
  --     aliases = optional_existing_client_names,
  --     status = optional_{starting,attached,reusing,failed}_messages,
  --     prepare = optional_function(start_server) }
  -- `server` is vim.lsp.Config with an executable array; we supply root_dir.
  -- `context` contains repository_root, project_root, language. prepare calls
  -- start_server once on the main event loop after successful setup. It runs
  -- only for the fallback server; personal clients own their setup lifecycle.
  local module_path = root .. "/editor/nvim/languages/" .. language .. ".lua"
  if vim.fn.filereadable(module_path) ~= 1 then
    vim.notify("No AoC LSP module for " .. language .. "; add " .. module_path .. " or use --user-config.", vim.log.levels.WARN)
    return
  end
  local ok, specification = pcall(function()
    local factory = assert(loadfile(module_path))()
    assert(type(factory) == "function", "language module must return a function(context)")
    local result = factory({ repository_root = root, project_root = project, language = language })
    assert(type(result) == "table" and type(result.server) == "table", "language module must provide server")
    assert(
      type(result.server.cmd) == "table" and type(result.server.cmd[1]) == "string",
      "server.cmd must be an executable array"
    )
    assert(result.prepare == nil or type(result.prepare) == "function", "prepare must be a function")
    assert(result.aliases == nil or type(result.aliases) == "table", "aliases must be an array")
    return result
  end)
  if not ok then
    vim.notify("Cannot load " .. module_path .. ": " .. tostring(specification), vim.log.levels.ERROR)
    return
  end

  local server = specification.server
  local status = specification.status or {}
  local name = "advent_" .. language
  local executable = vim.fs.basename(server.cmd[1])
  local aliases = {}
  for _, alias in ipairs(specification.aliases or {}) do
    aliases[alias] = true
  end
  aliases[executable] = true

  local function beneath(path, parent)
    path, parent = vim.fs.normalize(path), vim.fs.normalize(parent)
    return path == parent or path:sub(1, #parent + 1) == parent .. "/"
  end
  local function project_buffer(buffer)
    return vim.api.nvim_buf_is_valid(buffer) and beneath(vim.api.nvim_buf_get_name(buffer), project)
  end
  local function compatible(client)
    if client.name == name then
      return false
    end
    local command = client.config.cmd
    return aliases[client.name]
      or (type(command) == "table" and type(command[1]) == "string"
        and vim.fs.basename(command[1]) == executable)
  end
  local function covers_project(client)
    local buffer = vim.api.nvim_get_current_buf()
    if project_buffer(buffer) and (client.attached_buffers or {})[buffer] then
      return true
    end
    local target = project_buffer(buffer) and vim.api.nvim_buf_get_name(buffer) or project
    local client_root = client.root_dir or client.config.root_dir
    if type(client_root) == "string" and beneath(target, client_root) then
      return true
    end
    for _, folder in ipairs(client.workspace_folders or {}) do
      if folder.uri and beneath(target, vim.uri_to_fname(folder.uri)) then
        return true
      end
    end
    return false
  end
  local function personal_server()
    for alias in pairs(aliases) do
      if vim.lsp.is_enabled(alias) then
        return true
      end
    end
    -- Neovim 0.11 normally hides clients still initializing. Including them
    -- avoids launching a duplicate while a personal server is starting.
    for _, client in ipairs(vim.lsp.get_clients({ _uninitialized = true })) do
      if compatible(client) and covers_project(client) then
        return true
      end
    end
    return false
  end

  local group = vim.api.nvim_create_augroup("AdventProjectLsp", { clear = true })
  local announced = {}
  local closing = false
  vim.api.nvim_create_autocmd("VimLeavePre", {
    group = group,
    once = true,
    callback = function() closing = true end,
  })
  vim.api.nvim_create_autocmd("LspAttach", {
    group = group,
    callback = function(event)
      if not project_buffer(event.buf) then
        return
      end
      local client = vim.lsp.get_client_by_id(event.data.client_id)
      if not client then
        return
      end
      if status.attached and not announced[client.id] and (client.name == name or compatible(client)) then
        announced[client.id] = true
        vim.notify(status.attached)
      end
      -- A plugin may start its own server on InsertEnter or another later
      -- event. Yield to that client and stop only our fallback, never theirs.
      if (compatible(client) and covers_project(client)) or (client.name == name and personal_server()) then
        vim.lsp.enable(name, false)
        return
      end
      if client.name ~= name or not options.defaults then
        return
      end
      local function map(keys, action, description)
        vim.keymap.set("n", keys, action, { buffer = event.buf, desc = description })
      end
      map("K", vim.lsp.buf.hover, "Hover documentation")
      map("gd", vim.lsp.buf.definition, "Go to definition")
      map("gD", vim.lsp.buf.declaration, "Go to declaration")
      map("gr", vim.lsp.buf.references, "Find references")
      map("gi", vim.lsp.buf.implementation, "Go to implementation")
      map("<leader>rn", vim.lsp.buf.rename, "Rename symbol")
      map("<leader>ca", vim.lsp.buf.code_action, "Code actions")
      map("<leader>f", function()
        vim.lsp.buf.format({ bufnr = event.buf, async = true })
      end, "Format buffer")
      if client:supports_method("textDocument/completion") then
        vim.lsp.completion.enable(true, client.id, event.buf, { autotrigger = true })
        vim.keymap.set("i", "<C-Space>", vim.lsp.completion.get, { buffer = event.buf, desc = "Complete symbol" })
      end
    end,
  })

  local function enable_server()
    if personal_server() then
      if status.reusing then
        vim.notify(status.reusing)
      end
      return
    end
    if vim.fn.executable(server.cmd[1]) ~= 1 then
      vim.notify(
        server.cmd[1] .. " is missing from PATH. Enter nix develop .#" .. language .. " or install the language server.",
        vim.log.levels.WARN
      )
      return
    end
    server.root_dir = function(buffer, on_dir)
      if project_buffer(buffer) and not personal_server() then
        on_dir(project)
      end
    end
    if status.failed then
      local on_exit = server.on_exit
      server.on_exit = function(code, signal, client_id)
        if on_exit then
          on_exit(code, signal, client_id)
        end
        vim.schedule(function()
          if not closing and vim.lsp.is_enabled(name) and (code ~= 0 or not announced[client_id]) then
            vim.notify(status.failed .. " (exit " .. tostring(code) .. ", signal " .. tostring(signal) .. ")", vim.log.levels.ERROR)
          end
        end)
      end
    end
    vim.lsp.config(name, server)
    if status.starting then
      vim.notify(status.starting)
    end
    vim.lsp.enable(name)
  end
  local function setup_server()
    if personal_server() then
      if status.reusing then
        vim.notify(status.reusing)
      end
      return
    end
    if not specification.prepare then
      enable_server()
    else
      local prepared, message = pcall(specification.prepare, enable_server)
      if not prepared then
        vim.notify("Editor setup failed for " .. language .. ": " .. tostring(message), vim.log.levels.ERROR)
      end
    end
  end

  -- -c runs after the first buffer's FileType but before VimEnter. Enabling an
  -- LSP before VimEnter would miss that buffer in Neovim 0.11. Scheduling after
  -- VimEnter also lets normal startup plugins register their own clients first.
  if vim.v.vim_did_enter == 1 then
    vim.schedule(setup_server)
  else
    vim.api.nvim_create_autocmd("VimEnter", {
      group = group,
      once = true,
      callback = function()
        vim.schedule(setup_server)
      end,
    })
  end
end
