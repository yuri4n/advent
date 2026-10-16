# Editors and IDEs

The repository supplies native project metadata, shared formatting defaults, and optional VS Code tasks. Use the same project roots and commands in any editor that supports these languages.

## Load the development environment

Complete the [Nix and direnv setup](manual.md#set-up-the-tools), including explicit `direnv allow` approval for the root and the language directories you use. The base shell contains common utilities and Neovim; each language shell adds its compiler and language server. `aoc edit` selects the language environment and your preferred editor. It uses `--editor COMMAND` first, then nonblank `VISUAL`, nonblank `EDITOR`, and finally Neovim. GUI editors and their language extensions are installed separately on your machine.

For the multi-language VS Code workspace, select all toolchains from the repository root:

```sh
nix develop .#all --command code advent.code-workspace
```

For a single language, use its shell, such as `nix develop .#scala`, and open that project. Tasks, language servers, and debuggers need that environment in the editor process; enabling a hook only inside an integrated terminal may leave those other processes with an older `PATH`. Restart the editor after changing how it loads the environment. The shell also provides ripgrep, fd, fzf, jq, Git, curl, and Python for searches and small development tasks.

## Project roots and language servers

| Language | Open this project | Language service | Formatter |
| --- | --- | --- | --- |
| Rust solutions | `rust/Cargo.toml` | rust-analyzer or a Cargo-aware Rust IDE | `cargo fmt --all` |
| Rust toolkit | `toolkit/Cargo.toml` | rust-analyzer or a Cargo-aware Rust IDE | `cargo fmt` |
| Go | `go/` with `go.mod` | gopls or GoLand | `gofmt` |
| Elixir | `elixir/` with `mix.exs` | ElixirLS or an Elixir IDE plugin | `mix format` |
| Scala | `scala/` with `project.scala` | Metals with Scala CLI's BSP server | `scala-cli fmt .` |
| Haskell | `haskell/` with `cabal.project` | Haskell Language Server | Fourmolu using `fourmolu.yaml` |

Formatting commands run inside the corresponding project. `.editorconfig` defines shared indentation and newline conventions; native formatters decide language syntax formatting.

With manual Rust toolchains, install the standard development components:

```sh
rustup component add rust-src rustfmt clippy
```

The Nix Rust shell already includes these components and rust-analyzer.

If rust-analyzer cannot find the projects when the repository root alone is opened, add `rust/Cargo.toml` and `toolkit/Cargo.toml` as linked Cargo projects in your editor, or open the supplied workspace. The two workspaces serve separate purposes: puzzle crates and the orchestration CLI.

Go has one module, so `go.work` is unnecessary. Open `go/` to give gopls the correct module root. Elixir solutions use compiled `.ex` modules with `Advent.Day` declarations, so Mix and ElixirLS see them without executing standalone scripts. The native `mix aoc.run`, `mix aoc.test`, and `mix aoc.new` tasks can be used directly in an editor terminal; see the [Elixir language guide](manual.md#elixir).

Generate Scala's BSP metadata from the repository root on your own machine:

```sh
bash scripts/languages/scala.sh editor-setup
```

The helper records an executable Scala CLI launcher in the BSP metadata, including when Scala CLI uses a Java wrapper. Then import or reconnect the build in Metals. Generated `.bsp`, `.scala-build`, and `.metals` files are local machine data; do not commit their absolute paths.

Metals is the recommended Scala setup. Importing the build makes the pinned Scala compiler, Kyo dependency, and compiler options available to the language server. Ordinary `Day` solutions return effect-free `Answer < Any` computations, with optional Kyo parallelism and effects; `StreamingDay` supplies streaming input. See the [Scala language guide](manual.md#scala) for both APIs, inline samples, and native test commands. Reimport the build after changing dependency directives.

HLS must support your selected GHC version. Build the Cabal project first, then use automatic Cabal discovery. There is no committed `hie.yaml` pointing at a developer's machine.

## VS Code

Open `advent.code-workspace`. It includes the repository, toolkit, and each language as named folders, allowing extensions to locate their native project roots. Install only the recommended extensions for the languages you use.

Use **Tasks: Run Task**:

- **AoC: new day** prompts for year, day, and language.
- **AoC: run** selects a part and displays a timing.
- **AoC: test** checks sample assertions for the selected part.
- **AoC: watch** keeps the command running while you edit.
- **AoC: doctor** reports available tools.

These tasks call `aoc` from the editor's `PATH`. They use process arguments instead of interpolating prompt answers into shell commands. Their working directory is the named `repository` workspace folder, so open the workspace file to use them. Start the editor in the development environment as described above.

Compilation diagnostics come from the language extension and native compiler output. The generic tasks do not pretend that all five compilers share one problem matcher. Cancel a watch task through **Tasks: Terminate Task**.

No workspace setting stores a session cookie or tool installation path. New-day generation does not rewrite editor preferences.

## Zed

Install Zed and enable its `zed` command-line launcher, then open a day from anywhere in the checkout:

```sh
aoc edit 2021 1 --lang scala --editor zed
```

Some Linux packages name the launcher `zeditor`; use `--editor zeditor` for that installation.

The command passes both the native project directory and solution file to Zed. Opening the full project makes its shared harness and build metadata available for navigation and diagnostics. Zed 0.152 or later [forwards the command-line environment](https://github.com/zed-industries/zed/blob/main/docs/src/environment.md) even to an already-running instance, so launching through `aoc edit` supplies the selected Nix tools to the project.

Install the Zed language extensions needed for your languages. Scala uses [the Scala extension backed by Metals](https://github.com/scalameta/metals-zed). Before opening Scala in Zed, `aoc edit` prepares its BSP project metadata with the selected toolchain. Its first import may download dependencies and index the project. This integration needs no changes to your global Zed settings.

To use Zed for future `aoc edit` commands, set your preference in the shell:

```sh
export VISUAL='zed --wait'
```

`--wait` is optional; omit it if you want the terminal command to return after handing the project to Zed. `--editor zed` overrides a different shell preference for one command. Editor commands are parsed into an executable and quoted arguments and run directly. Shell pipelines, redirects, variable expansion, `~` expansion, and command substitution are not evaluated; use an executable on `PATH` or an absolute path.

## Neovim

Open an existing day from anywhere in the checkout:

```sh
aoc edit 2021 1 --lang scala --editor nvim
aoc edit 2022 1 --lang rust --editor nvim
```

The command starts Neovim normally, loads your personal configuration, then adds the AoC project and language-server integration. Neovim's usual configuration lookup applies, including `NVIM_APPNAME`; the working directory is the native language project. It selects the corresponding Nix shell when available, so Neovim and the language server see the same tools. A missing solution produces a `new` instruction instead of creating a file implicitly.

Choose the configuration mode that fits the session:

| Command option | Configuration |
| --- | --- |
| No extra flag | Your normal configuration plus AoC project/LSP integration; bundled defaults if startup has no existing configuration. |
| `--standalone` | The isolated [bundled configuration](../editor/nvim/init.lua). |
| `--user-config` | Your normal configuration alone, with no AoC overlay. |

```sh
aoc edit 2021 1 --lang scala --editor nvim
aoc edit 2021 1 --lang scala --editor nvim --standalone
aoc edit 2021 1 --lang scala --editor nvim --user-config
```

`--standalone` and `--user-config` are Neovim-only options and cannot be combined. Add `--no-nix` to any mode to use tools already on `PATH`.

The command reports when it is preparing a language environment, then identifies the solution and configuration mode being opened. The first Nix preparation can take time while tools download or build.

The AoC integration needs Neovim 0.11 or later and uses its built-in LSP client. It installs no plugins and does not modify your configuration files. With an existing configuration, the overlay preserves its options, runtime plugins, mappings, diagnostic display, and completion setup. It respects matching enabled or starting language servers; if your setup attaches its own matching server later, the overlay stops only its fallback server.

When a server is needed, the AoC module roots it at the complete native project, including shared harness code. For Scala, the module prepares local BSP metadata before starting Metals. An existing personal Metals setup keeps control of its own preparation. Editing remains available while AoC preparation runs.

The bundled defaults provide these shortcuts:

| Key | Action |
| --- | --- |
| `K` | Hover documentation. |
| `gd` / `gD` | Definition / declaration. |
| `gr` / `gi` | References / implementation. |
| `Space rn` | Rename symbol. |
| `Space ca` | Code action. |
| `Space f` | Format the current buffer. |
| `Space e` | Show diagnostics for the current line. |
| `Space q` | Open the diagnostics location list. |
| `Ctrl-Space` in insert mode | Request completion; supported servers also trigger it automatically. |

These mappings and built-in completion apply to standalone or unconfigured sessions; configured sessions keep their own mappings and completion. LSP navigation and editing actions become available when the server attaches. The bundled defaults show inline diagnostics and support manual formatting; they do not format or save automatically. Use ordinary Neovim commands such as `:w` to save and `:q` to quit.

Custom languages open with the same command; add a language server module below or use your own Neovim configuration.

### Diagnose editor startup or language-server problems

Use `:messages` to inspect startup errors and `:checkhealth vim.lsp` to inspect language-server setup. To see the configuration directory Neovim selected, run `:lua print(vim.fn.stdpath("config"))`; this is particularly useful with `NVIM_APPNAME`.

Check that the server is available in the selected environment, for example `:lua print(vim.fn.exepath("rust-analyzer"))` for Rust. An empty result means the executable is missing from the editor's `PATH`. Select the language's Nix shell or install that server for a manual toolchain. For Scala metadata errors, run `bash scripts/languages/scala.sh editor-setup` from the repository root and read its output.

Scala's first setup may download compiler dependencies and then import and index the project. AoC reports preparation, metadata readiness, Metals startup, and attachment as separate stages. Attachment establishes the language-server connection; build import and diagnostics may still be running. Use `:messages` and LSP health to check progress or failures. Scala's `???` placeholder is valid code and normally produces no compiler diagnostic. Running an unfinished part exposes that placeholder at runtime.

If a theme, plugin, or personal LSP setup fails with the selected Neovim version, compare `--standalone` and `--user-config` sessions. Those modes help distinguish a personal-configuration problem from an interaction with the AoC overlay. Existing configurations can have version-specific requirements; check their startup errors before changing settings.

### Add a Neovim language

Language server definitions live in `editor/nvim/languages/NAME.lua`. Adding a matching module makes it available to `aoc edit` without changing the shared initializer. For example, after registering Python and providing Pyright in its Nix shell, `editor/nvim/languages/python.lua` could contain:

```lua
return function()
  return {
    aliases = { "pyright" },
    server = {
      cmd = { "pyright-langserver", "--stdio" },
      filetypes = { "python" },
    },
  }
end
```

The function receives a context table containing `repository_root`, `project_root`, and `language` if it needs them. Return a `server` table using Neovim's standard `vim.lsp.Config` fields, including an executable command array. Optional `aliases` names let the overlay recognize existing server configurations. The shared overlay supplies the project root when it starts a fallback server; completion and keymaps are added only for standalone or unconfigured sessions.

A module may also return `prepare = function(start_server) ... end` for setup such as Scala's BSP refresh. It runs only when the AoC fallback is needed. Call `start_server` once on the main event loop after setup succeeds; use `vim.schedule_wrap` for an asynchronous `vim.system` callback. On failure, report an actionable message and leave the file available for editing. Optional `status` messages can describe `starting`, `attached`, and `reusing` stages. See [the Scala module](../editor/nvim/languages/scala.lua) for that pattern.

## Emacs, Helix, and other LSP clients

Configure your normal LSP integration to start the server from the project root in the table above. Use the installed toolchain from the same environment as your terminal. In a remote/cloud session, run the language server where the checkout and build tools live.

Bind `aoc` from the development environment to an editor task, terminal command, or test action:

```text
aoc test YEAR DAY --lang LANG --part 1
aoc run YEAR DAY --lang LANG --part 1 --time
```

Run these commands with a working directory inside the checkout. The CLI finds the repository by walking parent directories, while language servers still need their native project roots. With manual toolchains, point your local editor command at `/path/to/advent/aoc` to use the compatibility launcher instead; keep machine-specific paths out of committed configuration.

## JetBrains tools

Open the Cargo project in a Rust-capable IDE, `go/` in GoLand, and the corresponding native project for other supported language plugins. Scala requires an IDE/plugin that can import the Scala CLI/BSP project; plugin compatibility varies by IDE version. IntelliJ's analysis of Kyo effect types and macros has not been validated here; use Metals for the recommended Scala integration, and the Scala CLI compiler to check diagnostics. Use the terminal command or an external tool pointing at `aoc` when an IDE has no matching run configuration.

## Debugging

Debug support depends on the selected language, editor, and installed debug adapter. Use the native run target to reach your part function:

- Rust: select the year crate's binary or an inline sample test through the Rust extension/debugger.
- Go: use Delve with the `cmd/aoc` executable or a Go sample test.
- Elixir: use the Elixir extension's Mix debugging support where available.
- Scala: use a Metals run/debug target after BSP import.
- Haskell: use a compatible Haskell debugging integration or GHCi for the day's module.

For command-line runners, pass the same year/day/part arguments as the adapter and arrange standard input from `data/yYY/dDD/input.txt`. A native sample test is usually the easiest deterministic debug target.

The repository does not commit a universal `launch.json`: executable paths, debug adapters, and compiler support differ across platforms.

## What is verified

Twenty headless startup tests using Neovim 0.11.5 and a fixture LSP server pass. They cover personal configuration discovery and preservation, standalone defaults, initial-buffer attachment, diagnostics, existing and delayed personal servers, and recoverable startup failures. Run them from the repository root with `python3 editor/nvim/tests/run.py`; `--nvim /path/to/nvim` selects another Neovim executable. The Nix `editor` check is configured to run the same regressions.

Actual rust-analyzer attachment and navigation into the shared Rust harness were checked. With the revised startup, Metals 1.6.3 reported an intentional type error in the Scala 3.7.3/Kyo 0.19.0 project and resolved a definition into the shared `Day` harness. Scala's editor-setup helper successfully generated an executable BSP launcher. Zed launch arguments and native project preparation were checked; an interactive Zed GUI session was not available in the cloud environment.

The workspace and task files are JSON, use repository-relative paths, and delegate to the shared CLI. Native build/test validation is separate from a full interactive editor session. Completion, diagnostics, formatting plugins, and breakpoint support depend on the installed editor/server versions; they are not claimed to have been exercised in every editor.
