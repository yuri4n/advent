-- Explicit standalone mode: `aoc edit ... --standalone`.
local directory = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h")
dofile(directory .. "/defaults.lua")(true)
dofile(directory .. "/project.lua")({ defaults = true })
