# yurian's Advent of Code

My Advent of Code solutions in Rust, Go, Elixir, Scala, and Haskell, with a shared toolkit for creating, running, and checking both parts of a puzzle. Solution files stay focused on puzzle logic; each language uses its own idioms and native build tools.

[Solutions](#solutions) · [Getting started](#getting-started) · [Developer manual](docs/manual.md) · [Editors and IDEs](docs/editors.md)

## Solutions

- [ ] Year 2025 [[Scala](scala/src/main/scala/aoc/solutions/y2025/)].
  - [x] Day 1 [[Scala](scala/src/main/scala/aoc/solutions/y2025/Day01.scala)].
    - [x] Part I.
    - [x] Part II.
- [ ] Year 2024 [[Scala](scala/src/main/scala/aoc/solutions/y2024/)].
  - [x] Day 1 [[Scala](scala/src/main/scala/aoc/solutions/y2024/Day01.scala)].
  - [ ] Day 2 [[Scala](scala/src/main/scala/aoc/solutions/y2024/Day02.scala)].
- [ ] Year 2023 [[Elixir](elixir/lib/y23/)].
  - [x] Day 1 [[Elixir](elixir/lib/y23/d01.ex)].
- [ ] Year 2022 [[Rust](rust/years/y2022/)].
  - [x] Day 1 [[Rust](rust/years/y2022/src/d01.rs)].
- [ ] Year 2021 [[Haskell](haskell/src/Year2021/) | [Rust](rust/years/y2021/) | [Scala](scala/src/main/scala/aoc/solutions/y2021/)].
  - [x] Day 1 [[Scala](scala/src/main/scala/aoc/solutions/y2021/Day01.scala)].
  - [x] Day 2 [[Scala](scala/src/main/scala/aoc/solutions/y2021/Day02.scala)].
  - [x] Day 3 [[Scala](scala/src/main/scala/aoc/solutions/y2021/Day03.scala)].
  - [x] Day 4 [[Haskell](haskell/src/Year2021/Day04.hs)].
  - [x] Day 5 [[Haskell](haskell/src/Year2021/Day05.hs)].
  - [x] Day 6 [[Haskell](haskell/src/Year2021/Day06.hs)].
  - [x] Day 7 [[Rust](rust/years/y2021/src/d07.rs)].
  - [x] Day 8 [[Rust](rust/years/y2021/src/d08.rs)].
    - [x] Part I.
    - [ ] Part II.
  - [x] Day 9 [[Haskell](haskell/src/Year2021/Day09.hs)].
- [ ] Year 2020 [[Elixir](elixir/lib/y20/) | [Go](go/y20/)].
  - [x] Day 1 [[Go](go/y20/d01/solution.go)].
  - [x] Day 2 [[Elixir](elixir/lib/y20/d02.ex)].

## Getting started

Install [Nix and direnv, then enable your shell's direnv hook](docs/manual.md#set-up-the-tools). From the repository root, review [.envrc](.envrc) and run `direnv allow` once to approve it. At the next prompt, a base environment supplies the compiled `aoc` command, common utilities, and Neovim on `PATH`. Language commands select their toolchain as needed; entering a language directory also selects its environment after approving that directory's `.envrc`.

```sh
aoc doctor --lang rust
aoc list
aoc test 2022 1 --lang rust --part 2
aoc run 2022 1 --lang rust --part 2 --time
aoc new 2026 1 --lang rust
aoc edit 2026 1 --lang rust
# Implement a part and add its example before running the new day.
aoc watch 2026 1 --lang rust
```

Without direnv, enter the base environment with `nix develop`, a language environment with `nix develop .#rust`, or every toolchain with `nix develop .#all`. For an existing manual toolchain installation, the root `./aoc` launcher remains available and builds the CLI with Cargo. New solutions contain unfinished stubs, never generated puzzle answers.

Run `nix develop .#all --command bash ./scripts/check.sh` to verify the toolkit and existing inline examples with all five toolchains. No personal puzzle input is needed.

Real puzzle inputs live in ignored `data/yYY/dDD/input.txt` files. Existing local inputs are retained during migration. Fetch your own input with `aoc fetch YEAR DAY` after setting `AOC_SESSION` securely in your environment. Example inputs and typed expected answers live inline in native source or tests. Run one with `aoc run YEAR DAY --lang LANG --part 1 --example`; stdin, clipboard, and explicit input files are also supported.

- [Developer manual](docs/manual.md): setup, commands, input sources, offline use, timing, benchmarks, and adding a language.
- [Writing a solution](docs/solutions.md): shared stream-first processing, native parser libraries, typed examples, and IDE execution.
- [Editors and IDEs](docs/editors.md): project roots, language servers, formatting, run/test tasks, and debugging.
- [Design and migration](docs/architecture.md): accepted decisions, data contract, existing solutions, and limitations.

The documentation is maintained as Markdown in this repository; there is no published documentation website yet. The language guides below describe each language's solution shape, example checks, and native commands.

Open [advent.code-workspace](advent.code-workspace) for optional VS Code integration, or open the language project in your preferred editor. The build metadata is the source of truth; no editor is required.

| Language | Project | Build tool | Instructions |
| --- | --- | --- | --- |
| Rust | [rust/](rust/) | Cargo; one crate per year | [Rust guide](docs/manual.md#rust) |
| Go | [go/](go/) | Go modules | [Go guide](docs/manual.md#go) |
| Elixir | [elixir/](elixir/) | Mix tasks, NimbleParsec, and example macros | [Elixir guide](docs/manual.md#elixir) |
| Scala | [scala/](scala/) | Scala CLI and Kyo; optional parallelism and streaming | [Scala guide](docs/manual.md#scala) |
| Haskell | [haskell/](haskell/) | Cabal | [Haskell guide](docs/manual.md#haskell) |
