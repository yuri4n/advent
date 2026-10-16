# Developer manual

[Solution APIs and parser libraries](solutions.md) · [Editors and IDEs](editors.md) · [Architecture](architecture.md)

## Set up the tools

Install [Nix](https://nixos.org/download/) and [direnv](https://direnv.net/docs/installation.html). Nix needs `nix-command flakes` enabled. If necessary, add `extra-experimental-features = nix-command flakes` to `~/.config/nix/nix.conf`.

Enable your shell's direnv hook in its configuration:

| Shell | File | Hook |
| --- | --- | --- |
| Bash | `~/.bashrc` | `eval "$(direnv hook bash)"` |
| Zsh | `~/.zshrc` | `eval "$(direnv hook zsh)"` |
| Fish | `~/.config/fish/config.fish` | `direnv hook fish \| source` |

Review the repository's `.envrc`, then run `direnv allow`. At the next prompt the base environment supplies the compiled `aoc` command, common utilities, and Neovim. Language-targeted commands select their Nix toolchain automatically. Entering a language directory also selects its environment after approving that directory's `.envrc`.

```sh
aoc doctor --lang scala
aoc list
```

Without direnv, choose a shell explicitly:

```sh
nix develop            # base tools
nix develop .#scala    # Scala CLI, JDK, Metals
nix develop .#all      # all five toolchains
```

The other named shells are `rust`, `go`, `elixir`, and `haskell`. Rust includes rust-analyzer; Go includes gopls; Elixir includes ElixirLS; Haskell includes Cabal, HLS, and the project's parser packages. Nix pins tool versions, but native package managers still need their dependency caches populated.

If you edit the toolkit, `direnv reload` rebuilds the packaged CLI. **New source files must be visible to the Git-backed flake**: add them to Git before rebuilding the Nix package (intent-to-add is sufficient when you do not want to stage contents). `./aoc` remains the Cargo-based launcher for using the working tree directly.

### Existing manual toolchains

From the root, use `./aoc` instead of `aoc`. That launcher requires Bash and Cargo even when the selected solution is in another language. Install only the native tools you need:

| Project | Tools / metadata |
| --- | --- |
| Toolkit/Rust | Cargo; `rust-toolchain.toml`, Cargo manifests and locks |
| Go | Go 1.19+; `go/go.mod`, `go/go.sum` |
| Elixir | Elixir ~>1.14 and compatible OTP; `elixir/mix.exs`, `mix.lock` |
| Scala | Scala CLI, JDK 25+ (required by Kyo RC7); `scala/project.scala` |
| Haskell | GHC, Cabal; `haskell/advent.cabal`, `cabal.project` |

Install dependencies with the relevant native build once while connected. The parser libraries and solution contracts are described in [Writing a solution](solutions.md).

## The daily loop

```sh
aoc new 2026 1 --lang scala
aoc edit 2026 1 --lang scala
# Implement a part and add a typed inline example.
aoc run 2026 1 --lang scala --part 1 --example
aoc test 2026 1 --lang scala --part 1
aoc fetch 2026 1
aoc run 2026 1 --lang scala --part 1 --time
aoc watch 2026 1 --lang scala --part 1 --example
```

Only fetch released puzzles. Scaffolding future days is fine. Templates contain no puzzle answers and no available parts until you implement/register them. Existing incomplete parts remain unavailable.

## Commands

| Command | Purpose |
| --- | --- |
| `new YEAR DAY --lang LANG` | Create a native solution and build registration, without overwriting existing work. |
| `edit YEAR DAY --lang LANG [--editor COMMAND]` | Open an existing solution in its native project. |
| `run YEAR DAY --lang LANG [--part 1\|2\|both] [SOURCE] [--time]` | Execute the selected parts. |
| `test YEAR DAY --lang LANG [--part 1\|2\|both]` | Check inline expectations, using the same processing path as a run. |
| `bench YEAR DAY --lang LANG [--part 1\|2\|both] [SOURCE] [--iterations N] [--warmup N] [--wall]` | Compare native with/without-processing times, or process wall time. |
| `watch YEAR DAY --lang LANG [--part 1\|2\|both] [SOURCE] [--time]` | Rebuild/rerun after source or input changes. |
| `fetch YEAR DAY [--offline] [--force]` | Read cached personal input or download it. |
| `puzzle YEAR DAY [--offline] [--refresh]` | Display cached or downloaded puzzle text. |
| `doctor [--lang LANG]` | Check native tools; without a language, selects all toolchains. |
| `list` | Show registered languages and adapters. |
| `add-language NAME --extension EXT --run COMMAND --test COMMAND [--build COMMAND] [--stub SOURCE]` | Register trusted custom commands and a template. |

Both `2025` and `25` mean 2025. Days are 1–25. CLI part selection defaults to `both`; select an implemented part while the other is unavailable. Explicit CLI execution remains strict. Native IDE entry points instead run whatever is available and skip absent data/parts.

### Input sources

Choose at most one source:

| Source | Behavior |
| --- | --- |
| No flag | `data/yYY/dDD/input.txt` |
| `--input PATH` | File path, relative to the invoking shell |
| `--stdin` or `--input -` | Standard input |
| `--clipboard` | Read the clipboard once; never modify it |
| `--example` | First inline example, without requiring an expected answer |
| `--example=N` | Numbered inline example, starting at 1 |

```sh
printf 'L68\nR18\n' | aoc run 2025 1 --lang scala --part 1 --stdin
aoc run 2025 1 --lang scala --part 1 --clipboard
aoc run 2025 1 --lang scala --part 1 --example
aoc run 2023 1 --lang elixir --part 2 --example=2
```

Example execution prints actual answers to stdout and reports `PASS`, `FAIL`, or `UNCHECKED` on stderr for each selected part. A mismatch includes expected/actual values and exits nonzero. Input-only or part-specific examples without an expectation for the selected part are explicitly unchecked, not passed. `test` checks the declared expectations; an empty fixture list must not count as a successful test suite. Examples can differ between parts. Input is never silently trimmed to make an example pass; normalize a source literal explicitly when needed, such as Scala's `.stripMargin`. Processors receive byte-backed inputs and explicitly choose lazy byte, UTF-8 character, string-line, or nested character-line views; see [input views](solutions.md#input-views).

Files are connected directly to the child process's stdin. A single-part stdin run also streams directly. When stdin must be replayed for both parts, benchmarks, or watch, the CLI uses a private temporary file instead of retaining the complete input in RAM. Clipboard/example extraction also uses temporary-file replay. These files are cleaned up by the OS/library; native benchmark preparation is separate and intentionally uses memory.

Clipboard reading uses `pbpaste` on macOS, `wl-paste` on Wayland, or `xclip`/`xsel` on X11. If no reader is available, pipe input with `--stdin`. `AOC_CLIPBOARD_COMMAND` can specify a trusted executable and quoted arguments; it is parsed without shell evaluation. Clipboard integration tests use a mock reader and do not inspect your real clipboard.

Watch rebuilds examples after source edits and reopens file inputs, including replaced files. Stdin and clipboard are captured once for that watch session; changing the clipboard is not a watch trigger. Watch does not run assertions automatically.

## Editors and direct execution

`edit` chooses `--editor COMMAND`, then nonblank `VISUAL`, then `EDITOR`, then `nvim`. Arguments are parsed without a shell. Zed/VS Code receive the project and solution paths. Scala BSP metadata is prepared for external editors. Add `--no-nix` to use the current environment instead of selecting a language shell.

```sh
aoc edit 2025 1 --lang scala --editor zed
export VISUAL='zed --wait'
```

Neovim normally loads your own configuration and adds the repository overlay. `--user-config` uses only your personal configuration; `--standalone` uses the bundled complete setup. These flags are Neovim-only and mutually exclusive. See [the editor guide](editors.md) for server setup, keybindings, and troubleshooting.

For direct/native execution, available examples and existing input are run without fetching anything. Missing examples, missing input, and unavailable parts are skipped; implemented parts that fail are still errors.

| Language | Native/IDE entry point |
| --- | --- |
| Scala | Run the individual `aoc.solutions.yYYYY.DayDD` object (`KyoApp`). |
| Rust | Run the year binary with `auto DAY`, or no arguments for all its days. |
| Go | Run `cmd/aoc` with `auto YEAR DAY`, or no arguments for all registered days. |
| Elixir | `mix run -e 'Advent.Y25.D01.run()'` |
| Haskell | From `haskell/`, `runghc -isrc src/Year2021/Day06.hs` |

These entry points preserve native project structure rather than turn every module/package into a separate executable.

## Timing and benchmarks

```sh
aoc bench 2025 1 --lang scala --part 1 --example --iterations 10 --warmup 3
aoc bench 2025 1 --lang scala --part 1 --example --wall
```

Built-in adapters report:

- **With processing:** shared processing plus the selected part, including lazy parsing actually consumed by that part.
- **Without processing:** the part operating on already-prepared input.
- **Solve-only setup:** processing plus materialization/replay preparation, reported separately.

Each timing category reports minimum, median, mean, and maximum. Defaults are ten measured runs and one warmup per mode. Answers must stay identical across iterations and modes. Build time and runtime startup are excluded from native timings. Fixture extraction and raw-input buffering happen before measurement.

Solve-only benchmarking explicitly permits buffering. A parsed stream is consumed into replayable data; it is not sufficient to time construction of a lazy stream. Immutable prepared values may be reused; mutable data needs copies. Replay/copy construction is outside the solve-only timer. The [language guides](solutions.md) document each language's replay support and limitations. Ordinary execution stays streaming.

`--wall` retains the original benchmark: separate process launches, one initial build, and timings including runtime startup and input handling. Custom languages without native benchmark support use this mode automatically. `run --time` likewise reports process wall time: runtime startup + processing + solving, with the build excluded. It does not separate parsing from solving; use `aoc bench` for the two native timing modes.

These are practical iteration timings, not a statistical microbenchmark framework. JVM/BEAM warmup, GC, scheduling, caches, native answer formatting, and very small sample sizes affect results. Use appropriate native benchmark tools for detailed optimization; do not infer language rankings from these numbers.

## Inputs, examples, and offline use

Personal inputs and downloaded statements live in ignored `data/yYY/dDD/` directories. Fetching input requires your own `AOC_SESSION` cookie in a secure environment setting. Never place it in command arguments, source, editor settings, committed files, or chat. The toolkit does not submit answers.

`fetch` reuses cached input unless `--force` is supplied. `puzzle` stores `puzzle.html`; `--refresh` downloads newly unlocked text. `--offline` reads existing cache files only and cannot be combined with force/refresh. Normal runs and IDE entry points never download data automatically.

Offline data access is separate from offline dependency installation. Warm the language toolchain/dependency caches first. Scala supports `AOC_OFFLINE=1`; Cargo and Cabal also have native `--offline` options. Nix does not prefetch personal inputs or every native dependency.

## Native workflows

```sh
nix develop .#all --command bash ./scripts/check.sh
```

With toolchains already available, run `./scripts/check.sh`; use `AOC_OFFLINE=1` for Scala's populated cache. The checks require no personal inputs.

| Directory | Checks |
| --- | --- |
| `toolkit/` | `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` |
| `rust/` | `cargo test --workspace --locked` |
| `go/` | `go test ./...`, `go vet ./...` |
| `elixir/` | `mix test`, `mix format --check-formatted` |
| `scala/` | `scala-cli test . --server=false` |
| `haskell/` | `cabal test --offline --test-show-details=direct` |

### Rust

See [the Rust solution guide](solutions.md#rust) for winnow, typed examples, associated types, and registration.

### Go

See [the Go solution guide](solutions.md#go) for Participle, stream callbacks, typed fixtures, and native execution.

### Elixir

See [the Elixir solution guide](solutions.md#elixir) for NimbleParsec and `process/1`.

### Scala

See [the Scala solution guide](solutions.md#scala) for inner types, Kyo Parse, typed example cases, optional parts, and IDE execution. Kyo 1.0.0-RC7 and Scala 3.9.0 are pinned; the published RC7 jars require Java 25, provided by the Scala Nix shell. Repeated runs reuse the packaged jar when source contents, resources, adapter, and toolchain fingerprint are unchanged. Source additions/deletions also invalidate the cache. Delete `.aoc/build/scala/advent.jar` to force a fresh build.

### Haskell

See [the Haskell solution guide](solutions.md#haskell) for Megaparsec, lazy input, typed examples, and `NFData` benchmarking.

## Add a language

Commands are trusted local code. Register only commands you would run yourself:

```sh
aoc add-language python --extension py \
  --run 'python3 {file} {part}' \
  --test 'python3 {file} --test {part}' \
  --stub 'raise NotImplementedError("implement the puzzle parts")'
```

This updates `aoc.toml` and creates `templates/python/day.py`; `new` then creates `solutions/python/yYY/dDD.py`. Existing files are not overwritten. `{file}` is already shell-quoted. `{year}`, `{day}`, and `{part}` are also available. Run commands receive stdin. Test commands must reject failed/missing assertions. Registering a language does not invent its native runner or install its dependencies.

For a richer integration, provide an adapter script in `aoc.toml`:

- `build YEAR`
- `run YEAR DAY PART`
- `test YEAR DAY PART`
- `example YEAR DAY INDEX` — emit raw fixture bytes only
- `new YEAR DAY`
- `doctor`

An adapter may opt into `example_expectations = true` and implement `expect YEAR DAY PART INDEX`. It emits `AOC_EXPECT_V1` followed by a newline and the displayed expected answer, or `AOC_NO_EXPECT_V1` if no expectation exists. Index/part errors must exit nonzero. The CLI checks the selected answer without running the solver twice. Adapters without this capability report examples as `UNCHECKED`.

An adapter may opt into `native_bench = true`. Its `bench YEAR DAY PART ITERATIONS WARMUP` command reads input from stdin and emits `AOC_BENCH_V1`, setup nanoseconds, exactly `ITERATIONS` lines of `total_ns solve_ns`, then the answer (which may contain newlines). All errors/diagnostics go to stderr. Validate answer stability before emitting a successful report.

Add a matching `nix/languages/NAME.nix` to supply tools, and an optional `editor/nvim/languages/NAME.lua` for Neovim support. Shell names `base`, `default`, and `all` are reserved. An optional `editor_setup` argument array prepares external-editor project metadata; see [editor integration](editors.md#add-a-neovim-language).

## Troubleshooting

- Missing `aoc`: approve `.envrc`, enable the direnv hook, or enter `nix develop`.
- New flags are not recognized: the installed CLI is stale; rebuild/reload it, or use the working-tree launcher with Cargo available.
- A new Rust module is missing during Nix packaging: make new source files visible to Git before evaluating the flake.
- Missing input: fetch it explicitly or choose `--example`, `--stdin`, `--clipboard`, or `--input`.
- Unavailable part: select an implemented part. IDE automatic execution skips unavailable parts; explicit CLI runs do not.
- No assertions: add a typed example expectation. Merely supplying input is not a passing test.
- Solve-only replay is unsupported: provide the language-specific replay operation or use `--wall`.
- Missing compiler/server: run `aoc doctor --lang LANG` and check [editor setup](editors.md).
- Dependency downloads fail: verify network and CA configuration; do not disable TLS/checksum validation.
