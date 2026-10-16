-- UI defaults are used only without an existing init, or in explicit standalone
-- mode. The project overlay never applies these to a configured editor.
return function(isolated)
  if isolated then
    vim.opt.runtimepath = { vim.env.VIMRUNTIME }
    vim.opt.packpath = { vim.env.VIMRUNTIME }
    vim.g.loaded_remote_plugins = 1
  end
  vim.g.mapleader = " "
  vim.opt.number = true
  vim.opt.signcolumn = "yes"
  vim.opt.termguicolors = true
  vim.opt.ignorecase = true
  vim.opt.smartcase = true
  vim.opt.completeopt = { "menu", "menuone", "noselect" }
  vim.opt.shadafile = "NONE"
  vim.opt.undofile = false
  vim.diagnostic.config({ severity_sort = true, underline = true, virtual_text = true })
  vim.keymap.set("n", "<leader>e", vim.diagnostic.open_float, { desc = "Line diagnostics" })
  vim.keymap.set("n", "<leader>q", vim.diagnostic.setloclist, { desc = "List diagnostics" })
end
