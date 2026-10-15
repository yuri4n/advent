# Advent of Code

My Advent of Code solutions in Rust, Go, Elixir, Scala, and Haskell, with a shared toolkit for creating, running, and checking both parts of a puzzle. Solution files stay focused on puzzle logic; each language uses its own idioms and native build tools.

```sh
./aoc doctor
./aoc list
./aoc test 2022 1 --lang rust --part 2
./aoc run 2022 1 --lang rust --part 2 --time
./aoc new 2026 1 --lang rust
# Implement a part and add its sample before running the new day.
./aoc watch 2026 1 --lang rust
```

The launcher builds the Rust CLI with Cargo. Install Rust and the toolchain for the language you want to use; `doctor` reports missing tools. New solutions contain unfinished stubs, never generated puzzle answers.

With all five toolchains installed, run `./scripts/check.sh` to verify the toolkit and existing inline samples. No personal puzzle input is needed.

Real puzzle inputs live in ignored `data/yYY/dDD/input.txt` files. Existing local inputs are retained during migration. Fetch your own input with `./aoc fetch YEAR DAY` after setting `AOC_SESSION` securely in your environment. Sample inputs and expected answers live inline in native source or tests.

- [Developer manual](docs/manual.md): setup, all commands, offline use, samples, timing, benchmarks, and adding a language.
- [Editors and IDEs](docs/editors.md): project roots, language servers, formatting, run/test tasks, and debugging.
- [Design and migration](docs/architecture.md): accepted decisions, data contract, existing solutions, and limitations.

The documentation is maintained as Markdown in this repository; there is no published documentation website yet. The language guides below describe each language's solution shape, sample checks, and native commands.

Open [advent.code-workspace](advent.code-workspace) for optional VS Code integration, or open the language project in your preferred editor. The build metadata is the source of truth; no editor is required.

| Language | Project | Build tool | Instructions |
| --- | --- | --- | --- |
| Rust | [rust/](rust/) | Cargo; one crate per year | [Rust guide](docs/manual.md#rust) |
| Go | [go/](go/) | Go modules | [Go guide](docs/manual.md#go) |
| Elixir | [elixir/](elixir/) | Mix tasks and sample macros | [Elixir guide](docs/manual.md#elixir) |
| Scala | [scala/](scala/) | Scala CLI and Kyo; optional parallelism and streaming | [Scala guide](docs/manual.md#scala) |
| Haskell | [haskell/](haskell/) | Cabal | [Haskell guide](docs/manual.md#haskell) |
