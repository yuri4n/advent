-- Optional, self-contained editor for `aoc edit`. Requires Neovim 0.11+.
-- Language servers come from the selected Nix shell or the existing PATH.
-- This file does not install plugins or modify the user's Neovim configuration.
if vim.fn.has("nvim-0.11") == 0 then
  vim.notify("aoc edit needs Neovim 0.11 or newer; use the Nix shell or --user-config.", vim.log.levels.ERROR)
  return
end

vim.g.mapleader = " "
-- Load Neovim's bundled filetypes, EditorConfig, and plugins only. The user's
-- runtime plugins remain available when `aoc edit --user-config` is selected.
vim.opt.runtimepath = { vim.env.VIMRUNTIME }
vim.opt.packpath = { vim.env.VIMRUNTIME }
vim.g.loaded_remote_plugins = 1
vim.opt.number = true
vim.opt.signcolumn = "yes"
vim.opt.termguicolors = true
vim.opt.ignorecase = true
vim.opt.smartcase = true
vim.opt.completeopt = { "menu", "menuone", "noselect" }
-- Keep these sessions out of the user's normal command and file history.
vim.opt.shadafile = "NONE"
vim.opt.undofile = false

vim.diagnostic.config({ severity_sort = true, underline = true, virtual_text = false })
vim.keymap.set("n", "<leader>e", vim.diagnostic.open_float, { desc = "Line diagnostics" })
vim.keymap.set("n", "<leader>q", vim.diagnostic.setloclist, { desc = "List diagnostics" })

vim.api.nvim_create_autocmd("LspAttach", {
  group = vim.api.nvim_create_augroup("AdventLsp", { clear = true }),
  callback = function(event)
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

    local client = vim.lsp.get_client_by_id(event.data.client_id)
    if client and client:supports_method("textDocument/completion") then
      vim.lsp.completion.enable(true, client.id, event.buf, { autotrigger = true })
      vim.keymap.set("i", "<C-Space>", vim.lsp.completion.get, {
        buffer = event.buf,
        desc = "Complete symbol",
      })
    end
  end,
})

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

-- Add languages/<name>.lua alongside nix/languages/<name>.nix; init needs no edit.
-- Each Lua file returns function(context), producing:
--   { server = { cmd = { "server", "args" }, filetypes = { ... }, ... },
--     prepare = optional_function(start_server) }
-- `server` is a vim.lsp.Config with an executable command array. The shared
-- initializer supplies root_dir. `context` contains repository_root,
-- project_root, and language. An optional prepare hook calls start_server once
-- setup succeeds (on the main event loop), or reports a recoverable failure.
local module_path = root .. "/editor/nvim/languages/" .. language .. ".lua"
if vim.fn.filereadable(module_path) ~= 1 then
  vim.notify(
    "No bundled LSP configuration for " .. language .. "; add " .. module_path .. " or use --user-config.",
    vim.log.levels.WARN
  )
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
  return result
end)
if not ok then
  vim.notify("Cannot load " .. module_path .. ": " .. tostring(specification), vim.log.levels.ERROR)
  return
end
local server = specification.server

local function enable_server()
  if vim.fn.executable(server.cmd[1]) ~= 1 then
    vim.notify(
      server.cmd[1] .. " is missing from PATH. Enter nix develop .#" .. language .. " or install the language server.",
      vim.log.levels.WARN
    )
    return
  end
  -- The native project root includes the shared harness and every solution.
  -- Root discovery from just the day's file would miss year/workspace metadata.
  server.root_dir = function(_, on_dir)
    on_dir(project)
  end
  local name = "advent_" .. language
  vim.lsp.config(name, server)
  vim.lsp.enable(name)
end

if not specification.prepare then
  enable_server()
else
  local prepared, message = pcall(specification.prepare, enable_server)
  if not prepared then
    vim.notify("Editor setup failed for " .. language .. ": " .. tostring(message), vim.log.levels.ERROR)
  end
end
