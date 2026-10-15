# Developer manual

## Set up the tools

Run commands from the repository root unless a section says otherwise. The `aoc` launcher uses Bash and Cargo; Linux, macOS, and Windows through WSL are the intended shell environments. Rust is required for the CLI even when your solutions use another language.

```sh
./aoc doctor
./aoc doctor --lang rust
./aoc list
```

Install only the languages you plan to use. A doctor failure for an unused language does not prevent using another installed language.

| Project | Required tools | Native metadata |
| --- | --- | --- |
| Toolkit | Rust/Cargo | `toolkit/Cargo.toml`, `toolkit/Cargo.lock` |
| Rust | Rust/Cargo | `rust/Cargo.toml`, per-year manifests |
| Go | Go | `go/go.mod` |
| Elixir | Elixir and Erlang/OTP, Mix | `elixir/mix.exs` |
| Scala | Scala CLI, JDK 17+ | `scala/project.scala` |
| Haskell | GHC, Cabal; Python 3 for scaffolding | `haskell/advent.cabal`, `haskell/cabal.project` |
| Downloads | curl and network access | `AOC_SESSION` for authenticated requests |

Use the versions declared by each native project. Add rust-analyzer, gopls, ElixirLS, Metals, or Haskell Language Server for [editor support](editors.md).

## The daily loop

```sh
./aoc new 2026 1 --lang rust
./aoc fetch 2026 1
./aoc puzzle 2026 1
./aoc test 2026 1 --lang rust --part 1
./aoc run 2026 1 --lang rust --part 1 --time
./aoc watch 2026 1 --lang rust --part 1
```

Use the year and day of an already released puzzle when fetching. Creating a future day's stub does not require that the puzzle be published.

`new` writes a solution stub and the registrations required by its build tool. Implement one part, add inline samples from the puzzle, and run its tests. Select `--part 1` while part 2 is unfinished; select `--part both` once both are ready. Templates do not solve puzzles.

The built-in language names are `rust`, `go`, `elixir`, `scala`, and `haskell`. The shared CLI chooses the day, input, and part; your native tools remain usable directly.

## Commands

| Command | Purpose |
| --- | --- |
| `new YEAR DAY --lang LANG` | Create an unfinished solution and native build registration. |
| `run YEAR DAY --lang LANG --part 1\|2\|both [--input PATH] [--time]` | Run the selected part or both parts. |
| `test YEAR DAY --lang LANG [--part 1\|2\|both]` | Check the day's inline sample assertions. |
| `bench YEAR DAY --lang LANG --part 1\|2\|both [--iterations N] [--warmup N] [--input PATH]` | Run warmups followed by repeated timed executions. |
| `watch YEAR DAY --lang LANG [--part 1\|2\|both] [--input PATH] [--time]` | Run again after source/input changes; stop with Ctrl-C. |
| `fetch YEAR DAY [--offline] [--force]` | Read the local input cache or fetch your own input. |
| `puzzle YEAR DAY [--offline] [--refresh]` | Display the cached puzzle statement or download it. |
| `doctor [--lang LANG]` | Check required toolchain commands and local configuration. |
| `list` | List registered languages and adapters. |
| `add-language NAME --extension EXT --run COMMAND --test COMMAND [--build COMMAND] [--stub SOURCE]` | Register another language and its template. |

Run `./aoc --help` for the command syntax supported by the installed checkout.

Both `2022` and `22` select year 2022. Days are 1–25; `--part` defaults to `both`. Watch reruns the solution after changes, including central input edits; it does not run sample tests automatically. Run `test` separately when updating a sample.

## Inputs, samples, and offline use

Real inputs have one shared location, independent of language:

```text
data/
  y21/
    d01/
      input.txt
```

The CLI finds the repository through `aoc.toml` and supplies the input to the native adapter. Use an explicit input file when experimenting:

```sh
./aoc run 2022 1 --lang rust --part 2 --input /tmp/my-example.txt
```

Inputs and downloaded puzzle material are local caches, excluded from Git. Supply `AOC_SESSION` using your shell's secret manager or the cloud environment's secret settings. It is the value of your own Advent of Code session cookie. Do not put it in a command argument, a committed environment file, editor settings, or chat. The toolkit does not submit answers.

`fetch` reuses an existing input. `--force` requests a fresh copy. `puzzle` caches the statement as `data/yYY/dDD/puzzle.html`; `--refresh` downloads it again, useful after unlocking part 2. `--offline` only reads an existing cache and reports a missing file instead of downloading it. Offline mode cannot be combined with force/refresh. Public puzzle text can be fetched without a session; a session lets AoC return any part you have unlocked.

Offline puzzle/input access and offline dependency installation are different. Run each native build once with its dependencies available before relying on an offline session. Cargo, Go, Mix, Scala CLI, and Cabal keep their own compiler and dependency caches.

For cached Scala builds/tests, set `AOC_OFFLINE=1` when invoking the toolkit. Native Cargo and Cabal commands also support `--offline`. A cache miss still needs a connected preparation step; the toolkit does not fabricate inputs or dependencies.

Samples are inline with native source/tests. Expected answers belong to those assertions; there is no mandatory shared `answers.toml`. A missing expected part can be left unset while working on the other part. An empty generated template is unfinished work, not a passing sample suite.

## Native workflows

These commands are useful for checking project health or configuring an editor. For running a day, prefer `./aoc` so input paths and part selection stay consistent.

Run `./scripts/check.sh` from the repository root to check the toolkit, Rust harness, Go, Elixir, Cabal, and all three migrated Scala days together. This requires all five toolchains and their dependencies; it does not require real puzzle inputs. Use `AOC_OFFLINE=1 ./scripts/check.sh` to keep the Scala builds on their populated dependency cache.

| Project directory | Native command |
| --- | --- |
| `toolkit/` | `cargo test --locked` |
| `rust/` | `cargo test --workspace --locked` |
| `go/` | `go test ./...` |
| `elixir/` | `mix test` |
| `scala/` | `scala-cli run . --server=false --main-class aoc.Main -- test 2021 1` |
| `haskell/` | `cabal build all`, then `cabal test` |

Haskell's Cabal manifest is maintained directly; Stack and hpack are no longer part of the workflow. Scala uses Scala CLI directives in `project.scala`; sbt is no longer required.

### Rust

A day lives at `rust/years/yYYYY/src/dDD.rs`. It exports `part1(input: &str)`, `part2(input: &str)`, and `SAMPLES`. Each part can return any concrete type implementing `Display`. Use an ordinary typed parsing helper when useful; there is no mandatory parse trait.

```rust
pub const SAMPLES: &[aoc::Sample] = &[
    aoc::sample!("example", "sample input", part1 = "expected answer"),
];
```

Replace the placeholder input and answer with an actual puzzle sample. Add `part2 = "expected answer"` when that part is implemented. The year crate's `aoc::main! { 1 => d01 }` registers modules and provides native sample tests; `new` updates it automatically. Native `cargo test` checks supplied expectations. `aoc test --part both` additionally requires coverage for both parts.

Use `aoc::input!("../../../data/y21/d07/input.txt")` when you explicitly want to embed an input at compile time. The path is relative to the year crate's Cargo manifest. Ordinary harness builds consume standard input, so cloning the repository and compiling it does not require private inputs. If a part uses embedded input instead of its argument, input overrides and sample tests cannot change that embedded value; keep sample-tested logic parameterized by input.

```sh
cargo run --manifest-path rust/Cargo.toml -p aoc-y2021 -- test 7 2
cargo run --manifest-path rust/Cargo.toml -p aoc-y2021 -- run 7 2 < data/y21/d07/input.txt
```

### Go

A day lives at `go/yYY/dDD/solution.go`, in package `dDD`. Implement exported `Part1(input string)` and `Part2(input string)` functions. The registry infers their result types; an ordinary `parse` helper is optional. The generated stub panics until implemented.

Put the inline fixture in `sample_test.go`. Keep the sample test named `TestSamples`, with subtests named `part1` and `part2`, so CLI part filtering can find them. Add ordinary Go assertions inside each subtest; see [the migrated day 1 tests](../go/y20/d01/sample_test.go).

```sh
cd go
go test ./y20/d01 -run '^TestSamples$/^part1$'
go run ./cmd/aoc 2020 1 1 < ../data/y20/d01/input.txt
```

### Elixir

A day lives at `elixir/lib/yYY/dDD.ex` in a module such as `Advent.Y20.D02`. Implement `part1(input)` and `part2(input)` with optional ordinary parsing helpers. New modules raise until implemented.

Put a heredoc fixture in `test/yYY/dDD_test.exs` and test it with ExUnit. Tag each part's assertions with `@tag :part1` or `@tag :part2` to support CLI selection; see [the migrated day 2 tests](../elixir/test/y20/d02_test.exs).

```sh
cd elixir
mix test test/y20/d02_test.exs --only part1
mix aoc 2020 2 1 < ../data/y20/d02/input.txt
```

### Scala

A day lives at `scala/src/main/scala/aoc/solutions/yYYYY/DayDD.scala`. Its object extends `aoc.Day`, implements `part1(input: String)` and `part2(input: String)`, and supplies inline `samples`. A shared parsing helper is optional. Concrete answer types can be returned because the runner formats the result.

```scala
override val samples = Seq(
  Sample("sample input", expected1 = Some("expected answer"), expected2 = None)
)
```

Replace the placeholder input and answer with a real puzzle sample. `None` means no assertion for that part; it does not assert an empty answer. New-day stubs use `???`.

Native execution accepts standard input:

```sh
cd scala
scala-cli run . --server=false --main-class aoc.Main -- run 2021 1 1 < ../data/y21/d01/input.txt
```

### Haskell

A day lives in `haskell/src/YearYYYY/DayDD.hs`. It exports a parser, the two part functions, and `samples`. The parser converts `String` into that day's input type; each part returns a value with a `Show` instance. New-day stubs use `undefined`.

```haskell
samples = [Sample "sample input" (Just "expected answer") Nothing]
```

Replace the placeholder values with a real sample. `Nothing` leaves one part unasserted. The scaffold registers the module with Cabal and `AOC.Registry`; HLS can follow these ordinary imports.

```sh
cd haskell
cabal run advent-exe -- test 2021 9 1
cabal run advent-exe -- run 2021 9 1 < ../data/y21/d09/input.txt
```

## Timing and benchmarks

```sh
./aoc run 2022 1 --lang rust --part 2 --time
./aoc bench 2022 1 --lang rust --part 2 --warmup 2 --iterations 10
```

Use a completed part and the same input when comparing changes. The shared benchmark builds once, runs the warmups, and reports minimum, median, mean, and maximum wall time. Defaults are one warmup and ten measured runs. It also rejects answers that change between runs.

These measurements include process/runtime startup and input handling; they do not isolate a function. Initial build time is excluded. JVM/BEAM startup and operating-system caches affect the result. Use a native benchmarking library for detailed allocation analysis or stable microbenchmarks. Avoid comparing languages from these timings alone.

## Add a language

The CLI can register a language without changing its Rust source. Specify the extension, a run command, a test command, and a syntactically valid unfinished template. Commands are trusted local code: only register commands you would run yourself.

```sh
./aoc add-language python --extension py \
  --run 'python3 {file} {part}' \
  --test 'python3 {file} --test {part}' \
  --stub 'raise NotImplementedError("implement the puzzle parts")'
```

Registration updates `aoc.toml` and writes `templates/python/day.py`. Then `./aoc new 2026 1 --lang python` creates `solutions/python/y26/d01.py`. Existing language registrations and source files are not overwritten. Use `--build COMMAND` when compilation is needed before a run.

Command templates can use `{file}`, `{year}`, `{day}`, and `{part}`. `{file}` is already shell-quoted by the toolkit, so do not wrap that placeholder in extra quotes. Commands run from the repository root. The run command receives the input on standard input; arrange your native runner to select the requested part and print its answer. The test command must reject failed or missing samples and support `1`, `2`, and `both`. The example deliberately creates an unfinished program. You must implement its runner and tests before it can pass.

Adding a language provides orchestration and file generation. Install its compiler/interpreter and language server, then add any native dependency manifest or editor project data it requires. The built-in VS Code language picker is a static convenience; add your new language there or use the terminal.

## Troubleshooting

- A command is missing: run `./aoc doctor --lang LANG`, install that tool, and restart terminals/editors so they inherit the updated `PATH`.
- Input is missing: fetch it after configuring `AOC_SESSION`, or pass `--input PATH`. Test samples do not require the real input.
- A part is unfinished: select the implemented part explicitly. Migration preserves existing incomplete solutions.
- A sample command fails with no assertions: supply a real inline sample and its expected answer for that part.
- Rust compile-time input fails: check the path used by the embedding macro and ensure the file exists before compiling.
- HLS cannot load a component: use an HLS build compatible with the selected GHC and run `cabal build all`.
- Metals cannot import the build: run `scala-cli setup-ide . --server=false` from `scala/` and reconnect the build server.
- A download fails: verify the network connection and, for your input, the current session secret. Keep TLS and checksum validation enabled.
