# Developer manual

Language guides: [Rust](#rust) · [Go](#go) · [Elixir](#elixir) · [Scala with Kyo](#scala) · [Haskell](#haskell). See [Editors and IDEs](editors.md) for language servers, formatting, and run/debug setup.

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
| `scala/` | `scala-cli test . --server=false` |
| `haskell/` | `cabal build all`, then `cabal test` |

Haskell's Cabal manifest is maintained directly; Stack and hpack are no longer part of the workflow. Scala uses Scala CLI directives in `project.scala` and Kyo for effects; sbt is no longer required.

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

A day lives at `elixir/lib/yYY/dDD.ex` in a module such as `Advent.Y20.D02`. Use `Advent.Day` to declare the year/day and inline samples, and keep `part1(input)` and `part2(input)` as ordinary pure functions. Parsing helpers are optional. The generated module is unfinished:

```elixir
defmodule Advent.Y26.D01 do
  use Advent.Day, year: 2026, day: 1

  def part1(_input), do: raise("TODO: implement part 1")
  def part2(_input), do: raise("TODO: implement part 2")
end
```

Add a `sample` declaration inside that module with real puzzle input and expected answers:

```elixir
sample "sample input", part1: "expected answer"
```

Expected answers can use the same value types as the part functions, for example an integer. Omit `part2:` until that part is ready, then add its expectation to the same sample or another declaration. Heredocs work for multiline samples; see [the migrated day 2](../elixir/lib/y20/d02.ex). The shared ExUnit suite discovers the declarations, so each day needs only its solution module. `mix test` checks the supplied expectations together with the harness tests, so part 2 can remain unfinished while working on part 1. A day with no expectations fails explicitly. `mix aoc.test --part both` requires expectations for both parts; use `--part 1` while only that part is ready.

The native Mix tasks use the same year/day/part selection as the root CLI. `aoc.run` reads the central `data/yYY/dDD/input.txt` by default; use `--input PATH` to select another file or `--stdin` to pipe input:

```sh
cd elixir
mix aoc.new 2026 1
mix aoc.test 2020 2 --part 1
mix aoc.run 2020 2 --part 1
mix aoc.run 2020 2 --part 1 --stdin < ../data/y20/d02/input.txt
mix test
```

`mix aoc.new` creates one module containing the unfinished part functions and a commented sample declaration. `./aoc new`, `./aoc run`, and `./aoc test` remain available from the repository root.

### Scala

A day lives at `scala/src/main/scala/aoc/solutions/yYYYY/DayDD.scala`. Its object extends `Day[Part1, Part2]`, receives input text, and returns Kyo computations with typed answers. The scaffold starts with `Long` for both answers; change either type to match your result, for example `Day[Int, String]`.

Scala uses [Kyo 0.19.0](https://github.com/getkyo/kyo/tree/v0.19.0), pinned alongside Scala 3.7.3 in [project.scala](../scala/project.scala). Consult that version's API: it uses `IO` and `Resource`, while newer Kyo examples may use different names. The project enables Kyo's recommended value-discard checks and strict equality.

The generated solution is deliberately unfinished:

```scala
package aoc.solutions.y2026

import aoc.{Day, Sample}
import kyo.<

object Day01 extends Day[Long, Long] {
  def part1(input: String): Long < Any = ???
  def part2(input: String): Long < Any = ???
  override val samples: Seq[Sample] = Seq.empty
}
```

`Answer < Any` means an effect-free Kyo computation. Write an ordinary expression as the function body: Kyo lifts its value automatically, so pure solutions need no `IO` wrapper. Keep the return annotation `Answer < Any`; a plain `Answer` return annotation cannot override the `Day` contract. Parsing and algorithm helpers remain ordinary functions returning plain values, with no mandatory parse interface. See [the migrated day 1](../scala/src/main/scala/aoc/solutions/y2021/Day01.scala) for a complete example.

The shared Kyo application reads input, evaluates the computation, and prints its answer. Use `input.linesIterator` when line iteration is convenient; the standard `Day` input is already a buffered string. Either part can opt into effects by declaring `Answer < aoc.Puzzle.Effects`, which permits `Async`, `Abort[Throwable]`, and `Resource`, or a narrower effect type. Both ordinary thrown exceptions and typed failures reach the shared error boundary, which reports the failure on stderr and exits unsuccessfully.

Add inline samples to the object:

```scala
override val samples = Seq(
  Sample("sample input", expected1 = Some("expected answer"), expected2 = None)
)
```

Replace the placeholder input and answer with a real puzzle sample. Expected answers are strings, matching the returned value's `toString`. `None` means no assertion for that part; it does not assert an empty answer. Keep `--part 1` selected until part 2 has an implementation and a sample expectation.

Use the shared CLI from the repository root:

```sh
./aoc new 2026 1 --lang scala
./aoc test 2021 1 --lang scala --part 1
./aoc run 2021 1 --lang scala --part 1 --time
./aoc watch 2021 1 --lang scala --part 1
```

Native execution accepts standard input. Run the MUnit harness tests separately from a selected day's inline samples:

```sh
cd scala
scala-cli test . --server=false
scala-cli run . --server=false --main-class aoc.Main -- test 2021 1 1
scala-cli run . --server=false --main-class aoc.Main -- run 2021 1 1 < ../data/y21/d01/input.txt
```

For completion and diagnostics, import the Scala CLI build in Metals using the [BSP setup instructions](editors.md#project-roots-and-language-servers).

#### Optional parallel work

An ordinary `Day` can use Kyo's bounded parallel traversal when expensive work on separate items is independent. This small API example counts characters across lines; inside a day object:

```scala
import aoc.Puzzle
import kyo.{<, Async}

def part1(input: String): Long < Puzzle.Effects =
  Async.foreach(input.linesIterator.toVector, concurrency = 4)(line =>
    line.length.toLong
  ).map(_.sum)
```

The callback can return a pure value directly; `Async.foreach` handles scheduling. `concurrency = 4` bounds the number in flight. Choose a limit that fits the workload and available CPUs; scheduling overhead can make small calculations such as this count slower. Use parallelism when the independent calculation is expensive enough to benefit. The default scaffold stays effect-free until this is useful.

The runner handles supported effects for both `Day` and `StreamingDay`. Handle any additional effects or different error types before returning, and keep effect runners and unsafe execution out of solution files.

#### Streaming input

Use `StreamingDay[Part1, Part2]` when a part needs an input stream. It has the same effect capabilities as `Day`, with `aoc.Input` as the argument in place of `String`. Parts return `Answer < aoc.Puzzle.Effects` or a narrower type, such as `Answer < IO`.

`Input` provides three ways to consume the source:

| Access | Result | Behavior |
| --- | --- | --- |
| `input.text` | `String < IO` | Buffer the complete UTF-8 input. |
| `input.lines` | `Stream[String, IO]` | Decode UTF-8 and read lines as the stream is consumed. |
| `input.bytes` | `Stream[Byte, IO]` | Stream raw bytes using 4 KiB reads. |

This small API example counts lines and bytes; it is not a puzzle solution or the generated template:

```scala
import aoc.{Input, StreamingDay}
import kyo.{<, IO}

object InputCounts extends StreamingDay[Long, Long] {
  def part1(input: Input): Long < IO =
    input.lines.fold(0L)((count, _) => count + 1L)

  def part2(input: Input): Long < IO =
    input.bytes.fold(0L)((count, _) => count + 1L)
}
```

Each `Input` can be consumed once: choose text, lines, or bytes. The runner provides fresh input for each part and sample. Line streaming removes line terminators, including LF, CRLF, and lone CR, preserves interior empty lines, and does not invent an extra empty line after a final terminator. It uses bounded read-ahead, but a very long line still needs memory proportional to that line. The runner leaves borrowed standard input open. Inline `Sample` declarations and the CLI commands work for both `Day` and `StreamingDay`.

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
