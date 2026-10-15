# Editors and IDEs

The repository supplies native project metadata, shared formatting defaults, and optional VS Code tasks. Use the same project roots and commands in any editor that supports these languages.

## Project roots and language servers

| Language | Open this project | Language service | Formatter |
| --- | --- | --- | --- |
| Rust solutions | `rust/Cargo.toml` | rust-analyzer or a Cargo-aware Rust IDE | `cargo fmt --all` |
| Rust toolkit | `toolkit/Cargo.toml` | rust-analyzer or a Cargo-aware Rust IDE | `cargo fmt` |
| Go | `go/` with `go.mod` | gopls or GoLand | `gofmt` |
| Elixir | `elixir/` with `mix.exs` | ElixirLS or an Elixir IDE plugin | `mix format` |
| Scala | `scala/` with `project.scala` | Metals/BSP or a compatible Scala IDE | `scala-cli fmt .` |
| Haskell | `haskell/` with `cabal.project` | Haskell Language Server | Fourmolu using `fourmolu.yaml` |

Formatting commands run inside the corresponding project. `.editorconfig` defines shared indentation and newline conventions; native formatters decide language syntax formatting.

For Rust, install the standard development components:

```sh
rustup component add rust-src rustfmt clippy
```

If rust-analyzer cannot find the projects when the repository root alone is opened, add `rust/Cargo.toml` and `toolkit/Cargo.toml` as linked Cargo projects in your editor, or open the supplied workspace. The two workspaces serve separate purposes: puzzle crates and the orchestration CLI.

Go has one module, so `go.work` is unnecessary. Open `go/` to give gopls the correct module root. Elixir solutions use compiled `.ex` modules, so Mix and ElixirLS see them without executing standalone scripts.

Generate Scala's BSP metadata on your own machine:

```sh
cd scala
scala-cli setup-ide . --server=false
```

Then import or reconnect the build in Metals. Generated `.bsp`, `.scala-build`, and `.metals` files are local machine data; do not commit their absolute paths.

HLS must support your selected GHC version. Build the Cabal project first, then use automatic Cabal discovery. There is no committed `hie.yaml` pointing at a developer's machine.

## VS Code

Open `advent.code-workspace`. It includes the repository, toolkit, and each language as named folders, allowing extensions to locate their native project roots. Install only the recommended extensions for the languages you use.

Use **Tasks: Run Task**:

- **AoC: new day** prompts for year, day, and language.
- **AoC: run** selects a part and displays a timing.
- **AoC: test** checks sample assertions for the selected part.
- **AoC: watch** keeps the command running while you edit.
- **AoC: doctor** reports available tools.

These tasks call the same root `aoc` launcher as the terminal. They use process arguments instead of interpolating prompt answers into shell commands. Task paths refer to the named `repository` workspace folder, so open the workspace file to use them.

Compilation diagnostics come from the language extension and native compiler output. The generic tasks do not pretend that all five compilers share one problem matcher. Cancel a watch task through **Tasks: Terminate Task**.

No workspace setting stores a session cookie or tool installation path. New-day generation does not rewrite editor preferences.

## Neovim, Emacs, Helix, and other LSP clients

Configure your normal LSP integration to start the server from the project root in the table above. Use the installed toolchain from the same environment as your terminal. In a remote/cloud session, run the language server where the checkout and build tools live.

Bind the repository launcher to an editor task, terminal command, or test action:

```text
/path/to/advent/aoc test YEAR DAY --lang LANG --part 1
/path/to/advent/aoc run YEAR DAY --lang LANG --part 1 --time
```

Use the actual checkout path only in your local editor settings; committed configuration uses relative paths. The CLI's root lookup handles working-directory differences, while language servers still need their native project roots.

## JetBrains tools

Open the Cargo project in a Rust-capable IDE, `go/` in GoLand, and the corresponding native project for other supported language plugins. Scala requires an IDE/plugin that can import the Scala CLI/BSP project; plugin compatibility varies by IDE version. Use the terminal command or an external tool pointing at `aoc` when an IDE has no matching run configuration.

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

The workspace and task files are JSON, use repository-relative paths, and delegate to the shared CLI. Native build/test validation is separate from a full interactive editor session. Completion, diagnostics, formatting plugins, and breakpoint support depend on the installed editor/server versions; they are not claimed to have been exercised in every editor.
