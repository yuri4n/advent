#!/usr/bin/env python3
"""Headless startup regressions; requires Neovim 0.11+ and Python, no downloads.

Run: python3 editor/nvim/tests/run.py --nvim /path/to/nvim
The fixture is a real stdio LSP server, so tests exercise startup, attachment,
diagnostic rendering and server ownership rather than mocking Neovim's API.
"""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

TESTS = Path(__file__).resolve().parent
EDITOR = TESTS.parent
REPOSITORY = EDITOR.parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--nvim", default=os.environ.get("NVIM_TEST_BINARY", "nvim"))
arguments, remaining = parser.parse_known_args()
NVIM = shutil.which(arguments.nvim)
if not NVIM:
    parser.error(f"Neovim executable not found: {arguments.nvim}")

PERSONAL = r'''
vim.g.personal_init_loaded = true
vim.g.mapleader = ","
vim.opt.number = false
vim.opt.termguicolors = false
vim.opt.completeopt = { "menuone", "noinsert" }
vim.opt.runtimepath:append(vim.env.AOC_TEST_RUNTIME)
vim.opt.packpath:append(vim.env.AOC_TEST_RUNTIME)
vim.diagnostic.config({ virtual_text = false, severity_sort = false })
vim.keymap.set("n", "gd", ":let g:personal_definition = 1<CR>")
vim.keymap.set("n", "K", ":let g:personal_hover = 1<CR>")
vim.keymap.set("i", "<C-Space>", "<Nop>")
'''

PRESERVED = r'''
assert(vim.g.personal_init_loaded, "personal init was bypassed")
assert(vim.g.personal_plugin_loaded, "personal runtime plugin was not loaded")
assert(vim.g.mapleader == ",")
assert(not vim.o.number and not vim.o.termguicolors)
assert(vim.o.completeopt == "menuone,noinsert")
assert(vim.o.runtimepath:find(vim.env.AOC_TEST_RUNTIME, 1, true))
assert(vim.o.packpath:find(vim.env.AOC_TEST_RUNTIME, 1, true))
assert(vim.fn.maparg("gd", "n") == ":let g:personal_definition = 1<CR>")
assert(vim.fn.maparg("K", "n") == ":let g:personal_hover = 1<CR>")
assert(vim.fn.maparg("<C-Space>", "i") == "<Nop>")
assert(vim.diagnostic.config().virtual_text == false)
assert(vim.diagnostic.config().severity_sort == false)
assert(_G.test_completions == 0, "overlay changed personal completion")
'''

WAIT_FOR_FALLBACK = r'''
assert(vim.wait(6000, function()
  local clients = vim.lsp.get_clients({ bufnr = 0 })
  return #clients == 1 and clients[1].name == "advent_rust"
end, 20), "fallback did not attach to the initial file")
assert(vim.lsp.get_clients({ bufnr = 0 })[1].root_dir == vim.env.AOC_PROJECT_ROOT)
'''

PERSONAL_SERVER = r'''
vim.lsp.config("rust_analyzer", {
  cmd = { vim.env.AOC_TEST_SERVER, "--delay", "0.3", "--tag", "personal" },
  filetypes = { "rust" },
  root_dir = vim.env.AOC_PROJECT_ROOT,
  settings = { personal_setting = "kept" },
  on_attach = function() vim.g.personal_on_attach = true end,
})
vim.lsp.enable("rust_analyzer")
'''


class StartupTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="advent-nvim-test-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.environment = os.environ.copy()
        for variable in ("VIMINIT", "EXINIT", "MYVIMRC", "NVIM", "NVIM_LISTEN_ADDRESS"):
            self.environment.pop(variable, None)
        for name in ("CONFIG_HOME", "DATA_HOME", "STATE_HOME", "CACHE_HOME", "RUNTIME_DIR"):
            directory = self.root / name.lower()
            directory.mkdir(mode=0o700)
            self.environment[f"XDG_{name}"] = str(directory)
        self.system = self.root / "system"
        self.system.mkdir()
        self.environment["XDG_CONFIG_DIRS"] = str(self.system)
        self.environment["NVIM_APPNAME"] = "advent-test"
        self.project = self.root / "project"
        self.project.mkdir()
        self.source = self.project / "main.rs"
        self.source.write_text("fn main() { type_error; }\n")
        (self.project / "Cargo.toml").write_text('[package]\nname="fixture"\nversion="0.1.0"\n')
        self.runtime = self.root / "personal-runtime"
        (self.runtime / "plugin").mkdir(parents=True)
        (self.runtime / "plugin" / "personal.lua").write_text("vim.g.personal_plugin_loaded = true\n")
        self.binary = self.root / "bin"
        self.binary.mkdir()
        self.server = self.binary / "rust-analyzer"
        shutil.copyfile(TESTS / "lsp_fixture.py", self.server)
        self.server.chmod(0o755)
        self.environment.update({
            "PATH": str(self.binary) + os.pathsep + self.environment.get("PATH", ""),
            "AOC_ROOT": str(REPOSITORY),
            "AOC_PROJECT_ROOT": str(self.project),
            "AOC_EDITOR_LANGUAGE": "rust",
            "AOC_NVIM_BOOTSTRAP": str(EDITOR / "bootstrap.lua"),
            "AOC_TEST_SERVER": str(self.server),
            "AOC_TEST_RUNTIME": str(self.runtime),
            "AOC_TEST_LSP_LOG": str(self.root / "lsp.jsonl"),
            "AOC_TEST_DRIVER": str(TESTS / "driver.lua"),
            "AOC_TEST_ASSERTIONS": str(self.root / "assertions.lua"),
            "AOC_TEST_PERSONAL": str(self.root / "personal.lua"),
            "AOC_TEST_NAME": self._testMethodName,
            "NVIM_LOG_FILE": str(self.root / "nvim.log"),
        })

    def personal(self, kind="lua", extra=""):
        (self.root / "personal.lua").write_text(PERSONAL + extra)
        app = self.environment["NVIM_APPNAME"]
        directory = Path(self.environment["XDG_CONFIG_HOME"]) / app
        if kind == "config_dirs":
            # Neovim 0.11 searches system init files under nvim even when
            # NVIM_APPNAME selects a different per-user configuration.
            directory = self.system / "nvim"
        if kind == "sysinit":
            self.environment["NVIM_APPNAME"] = "nvim"
            directory = self.system / "nvim"
        directory.mkdir(parents=True, exist_ok=True)
        command = "lua dofile(vim.env.AOC_TEST_PERSONAL)"
        if kind in ("VIMINIT", "EXINIT"):
            self.environment[kind] = command
        elif kind in ("vim", "sysinit", "config_dirs"):
            (directory / ("sysinit.vim" if kind == "sysinit" else "init.vim")).write_text(command + "\n")
        else:
            (directory / "init.lua").write_text("dofile(vim.env.AOC_TEST_PERSONAL)\n")

    def run_editor(self, assertions, mode="overlay"):
        (self.root / "assertions.lua").write_text(assertions)
        trace = r'''
_G.test_notices = {}
vim.notify = function(message) table.insert(_G.test_notices, tostring(message)) end
_G.test_completions = 0
local completion = vim.lsp.completion.enable
vim.lsp.completion.enable = function(...)
  _G.test_completions = _G.test_completions + 1
  return completion(...)
end
'''
        command = [NVIM, "--headless", "--cmd", "lua " + trace]
        if mode == "overlay":
            command += ["-c", "lua dofile(vim.env.AOC_NVIM_BOOTSTRAP)"]
        elif mode == "standalone":
            command += ["-u", str(EDITOR / "init.lua")]
        command += ["-c", "lua dofile(vim.env.AOC_TEST_DRIVER)", "--", str(self.source)]
        result = subprocess.run(command, cwd=self.project, env=self.environment, capture_output=True,
                                text=True, timeout=20)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        log = self.root / "lsp.jsonl"
        return [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []

    def scala(self):
        self.source = self.project / "Day01.scala"
        self.source.write_text("object Day01 { val answer: Int = type_error }\n")
        self.environment["AOC_EDITOR_LANGUAGE"] = "scala"
        self.server = self.binary / "metals"
        shutil.copyfile(TESTS / "lsp_fixture.py", self.server)
        self.server.chmod(0o755)
        self.environment["AOC_TEST_SERVER"] = str(self.server)
        launcher = self.root / "scala cli"
        launcher.write_text(f"#!{sys.executable}\n" + '''
import json, os, pathlib, sys
assert sys.argv[1:] == ["--initial-launcher-path", str(pathlib.Path(__file__).absolute()), "setup-ide", ".", "--server=false"], sys.argv
pathlib.Path("setup-called.json").write_text(json.dumps(sys.argv))
if os.environ.get("AOC_TEST_SETUP_FAIL"):
    print("Fixture build metadata failure", file=sys.stderr)
    sys.exit(17)
pathlib.Path(".bsp").mkdir(exist_ok=True)
pathlib.Path(".bsp/scala-cli.json").write_text(json.dumps({"argv": [sys.argv[2], "bsp", "."]}))
''')
        launcher.chmod(0o755)
        self.environment["SCALA_CLI"] = str(launcher)

    def test_scala_launcher_progress_and_visible_diagnostics(self):
        self.scala()
        events = self.run_editor(r'''
assert(vim.wait(6000, function() return #vim.diagnostic.get(0) == 1 end, 20))
assert(vim.lsp.get_clients({ bufnr = 0 })[1].name == "advent_scala")
local messages = table.concat(_G.test_notices, "\n")
local preparing = assert(messages:find("Preparing Scala CLI", 1, true))
local prepared = assert(messages:find("build metadata ready", 1, true))
local starting = assert(messages:find("Starting Metals", 1, true))
local attached = assert(messages:find("Metals attached", 1, true))
assert(preparing < prepared and prepared < starting and starting < attached)
assert(vim.diagnostic.config().virtual_text == true)
''')
        bsp = json.loads((self.project / ".bsp" / "scala-cli.json").read_text())
        self.assertEqual(bsp["argv"][0], self.environment["SCALA_CLI"])
        self.assertIn("metals/doctorVisibilityDidChange", [event["method"] for event in events])

    def test_scala_setup_failure_is_visible_and_editing_still_works(self):
        self.scala()
        self.environment["AOC_TEST_SETUP_FAIL"] = "1"
        events = self.run_editor(r'''
assert(vim.wait(5000, function()
  return table.concat(_G.test_notices, "\n"):find("setup%-ide failed") ~= nil
end, 20))
assert(table.concat(_G.test_notices, "\n"):find("Fixture build metadata failure", 1, true))
assert(#vim.lsp.get_clients({ bufnr = 0 }) == 0)
assert(not vim.lsp.is_enabled("advent_scala"))
assert(vim.bo.filetype == "scala" and vim.bo.modifiable)
''')
        self.assertEqual(events, [])

    def test_metals_exit_replaces_starting_message_with_failure(self):
        self.scala()
        self.server.write_text(f"#!{sys.executable}\nraise SystemExit(7)\n")
        self.run_editor(r'''
assert(vim.wait(5000, function()
  return table.concat(_G.test_notices, "\n"):find("Metals stopped", 1, true) ~= nil
end, 20), "Metals exited but the UI remained at Starting Metals")
assert(table.concat(_G.test_notices, "\n"):find("checkhealth vim.lsp", 1, true))
''')

    def test_personal_metals_owns_its_setup_and_completion(self):
        self.scala()
        self.personal(extra=r'''
vim.lsp.config("metals", {
  cmd = { vim.env.AOC_TEST_SERVER, "--tag", "personal" },
  filetypes = { "scala" },
  root_dir = vim.env.AOC_PROJECT_ROOT,
  on_attach = function() vim.g.personal_on_attach = true end,
})
vim.lsp.enable("metals")
''')
        self.run_editor(r'''
assert(vim.wait(5000, function() return vim.g.personal_on_attach end, 20))
assert(#vim.lsp.get_clients({ bufnr = 0 }) == 1)
assert(not vim.lsp.is_enabled("advent_scala"))
''' + PRESERVED)
        self.assertFalse((self.project / "setup-called.json").exists(), "overlay reran a personal server's setup")

    def test_no_init_attaches_initial_buffer_and_renders_diagnostics(self):
        self.run_editor(WAIT_FOR_FALLBACK + r'''
assert(vim.wait(5000, function() return #vim.diagnostic.get(0) == 1 end, 20))
assert(vim.diagnostic.get(0)[1].message:find("Intentional type error", 1, true))
assert(vim.o.number and vim.g.mapleader == " ")
assert(vim.diagnostic.config().virtual_text == true)
assert(_G.test_completions == 1)
assert(vim.fn.maparg("gd", "n") ~= "")
local rendered = false
for _, mark in ipairs(vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, { details = true })) do
  for _, chunk in ipairs(mark[4].virt_text or {}) do
    rendered = rendered or chunk[1]:find("Intentional type error", 1, true) ~= nil
  end
end
assert(rendered, "diagnostic was received but not rendered inline")
''')

    def test_personal_init_lua_preserves_plugins_options_maps_and_completion(self):
        self.personal()
        self.run_editor(WAIT_FOR_FALLBACK + PRESERVED)

    def test_personal_init_vim(self):
        self.personal("vim")
        self.run_editor(WAIT_FOR_FALLBACK + PRESERVED)

    def test_nvim_appname(self):
        self.environment["NVIM_APPNAME"] = "my-neovim"
        self.personal()
        self.run_editor(WAIT_FOR_FALLBACK + PRESERVED)

    def test_xdg_config_dirs(self):
        self.personal("config_dirs")
        self.run_editor(WAIT_FOR_FALLBACK + PRESERVED)

    def test_viminit(self):
        self.personal("VIMINIT")
        self.run_editor(WAIT_FOR_FALLBACK + PRESERVED)

    def test_exinit(self):
        self.personal("EXINIT")
        self.run_editor(WAIT_FOR_FALLBACK + PRESERVED)

    def test_system_init(self):
        self.personal("sysinit")
        self.run_editor(WAIT_FOR_FALLBACK + PRESERVED)

    def test_inherited_myvimrc_does_not_hide_missing_config(self):
        self.environment["MYVIMRC"] = str(self.root / "different-editor-init.lua")
        self.run_editor(WAIT_FOR_FALLBACK + 'assert(vim.o.number and vim.g.mapleader == " ")')

    def test_standalone_bypasses_personal_init(self):
        self.personal()
        self.run_editor(WAIT_FOR_FALLBACK + r'''
assert(not vim.g.personal_init_loaded)
assert(not vim.g.personal_plugin_loaded)
assert(vim.o.number and vim.g.mapleader == " ")
assert(not vim.o.runtimepath:find(vim.env.AOC_TEST_RUNTIME, 1, true))
''', mode="standalone")

    def test_user_config_only_has_no_overlay(self):
        self.personal()
        events = self.run_editor(PRESERVED + 'assert(#vim.lsp.get_clients({ bufnr = 0 }) == 0)', mode="personal")
        self.assertEqual(events, [])

    def test_enabled_personal_server_is_preserved_without_duplicate(self):
        self.personal(extra=PERSONAL_SERVER)
        events = self.run_editor(r'''
assert(vim.wait(5000, function() return vim.g.personal_on_attach end, 20))
assert(#vim.lsp.get_clients({ bufnr = 0 }) == 1)
assert(vim.lsp.get_clients({ bufnr = 0 })[1].name == "rust_analyzer")
assert(vim.lsp.config.rust_analyzer.settings.personal_setting == "kept")
assert(not vim.lsp.is_enabled("advent_rust"))
''' + PRESERVED)
        self.assertEqual([event["tag"] for event in events if event["method"] == "initialize"], ["personal"])

    def test_starting_custom_named_server_is_recognized_by_command(self):
        self.personal(extra=r'''
vim.api.nvim_create_autocmd("FileType", {
  pattern = "rust",
  callback = function()
    vim.lsp.start({ name = "my_rust", cmd = { vim.env.AOC_TEST_SERVER, "--delay", "0.5", "--tag", "personal" },
      root_dir = vim.env.AOC_PROJECT_ROOT })
  end,
})
''')
        events = self.run_editor(r'''
assert(vim.wait(5000, function()
  local clients = vim.lsp.get_clients({ bufnr = 0 })
  return #clients == 1 and clients[1].name == "my_rust"
end, 20))
assert(not vim.lsp.is_enabled("advent_rust"))
''' + PRESERVED)
        self.assertEqual([event["tag"] for event in events if event["method"] == "initialize"], ["personal"])

    def test_vimenter_lazy_server_takes_precedence(self):
        self.personal(extra='vim.api.nvim_create_autocmd("VimEnter", { once=true, callback=function()\n'
                      + PERSONAL_SERVER + '\nend })\n')
        events = self.run_editor(r'''
assert(vim.wait(5000, function() return vim.g.personal_on_attach end, 20))
assert(not vim.lsp.is_enabled("advent_rust"))
''' + PRESERVED)
        self.assertEqual([event["tag"] for event in events if event["method"] == "initialize"], ["personal"])

    def test_server_for_another_project_does_not_suppress_current_file(self):
        other = self.project / "other-year"
        other.mkdir()
        (other / "main.rs").write_text("fn main() {}\n")
        self.personal(extra=r'''
vim.api.nvim_create_autocmd("VimEnter", { once = true, callback = function()
  local other = vim.env.AOC_PROJECT_ROOT .. "/other-year"
  local buffer = vim.fn.bufadd(other .. "/main.rs")
  vim.fn.bufload(buffer)
  vim.lsp.start({ name = "other_year_rust", cmd = { vim.env.AOC_TEST_SERVER, "--tag", "other-project" },
    root_dir = other }, { bufnr = buffer })
end })
''')
        events = self.run_editor(WAIT_FOR_FALLBACK + PRESERVED + r'''
assert(vim.wait(5000, function() return #vim.lsp.get_clients() == 2 end, 20))
assert(vim.lsp.is_enabled("advent_rust"))
''')
        self.assertEqual(sorted(event["tag"] for event in events if event["method"] == "initialize"),
                         ["fallback", "other-project"])

    def test_later_personal_server_replaces_only_fallback(self):
        self.personal()
        events = self.run_editor(WAIT_FOR_FALLBACK + PERSONAL_SERVER + r'''
assert(vim.wait(5000, function()
  local clients = vim.lsp.get_clients({ bufnr = 0 })
  return #clients == 1 and clients[1].name == "rust_analyzer"
end, 20), "AoC fallback remained after the personal server attached")
assert(vim.g.personal_on_attach)
assert(not vim.lsp.is_enabled("advent_rust"))
''' + PRESERVED)
        self.assertEqual(sorted(event["tag"] for event in events if event["method"] == "initialize"),
                         ["fallback", "personal"])


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0], *remaining], verbosity=2)
