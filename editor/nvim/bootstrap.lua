-- Loaded with -c after Neovim has sourced its normal configuration and opened
-- the solution. Neovim owns XDG_CONFIG_HOME, NVIM_APPNAME, and init discovery.
local directory = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h")
local function has_configuration()
  if (vim.env.VIMINIT or "") ~= "" or (vim.env.EXINIT or "") ~= "" then
    return true
  end
  local user_init = vim.env.MYVIMRC
  for _, script in ipairs(vim.fn.getscriptinfo()) do
    -- Check what was actually sourced: MYVIMRC can be inherited from a parent
    -- Neovim even when this NVIM_APPNAME has no configuration of its own.
    if (user_init and vim.fs.normalize(script.name) == vim.fs.normalize(user_init))
      or vim.fs.basename(script.name) == "sysinit.vim" then
      return true
    end
  end
  return false
end

local defaults = not has_configuration()
if defaults then
  -- The normal runtime is already loaded; do not replace it after startup.
  dofile(directory .. "/defaults.lua")(false)
end
dofile(directory .. "/project.lua")({ defaults = defaults })
