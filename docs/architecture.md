# Design and migration

## Decisions

| Area | Decision |
| --- | --- |
| Real inputs | Keep local copies in ignored central `data/yYY/dDD/input.txt`; fetch your own input. |
| CLI | Rust binary behind the root `aoc` launcher. |
| Build tools | Cargo, Go modules, Mix, Scala CLI, and Cabal. |
| Scala effects | Effect-free Kyo computations by default; optional bounded parallelism, effects, and streaming with a shared application boundary. |
| Existing solutions | Move them onto the shared run/test contract while retaining their algorithms and unfinished parts. |
| Languages | Five built-in adapters, plus `add-language` for a custom command/template registration. |
| Samples | Inline native samples/tests rather than a mandatory shared answers file. |
| Rust layout | One crate per year and a shared harness. |
| Rust registration | Explicit macro/registry rather than link-time collection. |
| Parsing | Support idiomatic parsing in each language; share parsed values between parts where the native harness supports it. |
| Development loop | Shared watch command, timing, and repeatable process benchmarks. |
| Puzzle material | Fetch and cache statements for terminal reading; explicit offline mode. |
| Diagnostics | Toolchain doctor. |
| Submission | No automatic answer submission. |
| Editors | Native project metadata and language servers, with optional shared VS Code tasks. |

Explicit Rust registration makes included days visible to Cargo and editors without a link-time collector or platform-specific linker behavior. The scaffolder updates the registration so the day file remains focused on puzzle logic.

The contract is shared; API syntax is idiomatic. Rust macros and Haskell/Scala/Elixir language features need not be reproduced as Go boilerplate.

## Execution contract

The root `aoc.toml` identifies the checkout and registers language adapters. A command resolves year, day, language, part, and input, then delegates building/testing/running to that adapter. The CLI captures command failures and keeps compiler/test output visible.

Real input is independent of the implementation language. Native adapters consume the chosen input through their runner; compile-time embedding remains available where the language supports it. Embedding an input is a build dependency, so the file must exist before that target is compiled.

Inline samples keep the input and expected answer close to the code being checked. Only supplied expectations are assertions. Selecting an unfinished part or a day with no relevant sample assertions must not be interpreted as successful validation.

Scala days implement `Day[Part1, Part2]` with functions from `String` to Kyo computations. The default `Answer < Any` annotation permits an ordinary pure expression without an `IO` wrapper, while parsing and algorithm helpers return plain values. A part may opt into `Async`, `Abort[Throwable]`, and `Resource`, including bounded parallel work, without changing its input shape. The shared `KyoApp` owns input and evaluation. `StreamingDay[Part1, Part2]` offers the same effect capabilities with single-pass `Input` access through buffered text, UTF-8 line streams, or raw byte streams. The runner supplies fresh input for each part and sample, and leaves borrowed standard input open. Successful runs print only the answer to stdout; typed failures and exceptions go to stderr with a nonzero exit status. Scala 3.7.3 and Kyo 0.19.0 are pinned together in `scala/project.scala`.

The toolkit does not infer puzzle answers, fill in missing algorithms, or submit results. `new` creates explicit unfinished stubs.

## Migration scope

The old language-specific harnesses used inconsistent file paths and hard-coded day selection. The shared adapters give existing days consistent selection, input resolution, and access to sample checks. Source organization follows the native language:

- Rust uses year crates with day modules.
- Go uses day packages in one module.
- Elixir uses compiled modules with `Advent.Day` metadata and inline samples; shared Mix tasks handle scaffolding, execution, and sample checks.
- Scala uses Scala CLI, Kyo effects, and discoverable year/day objects.
- Haskell uses Cabal and an explicit registry of modules.

Old solutions can still contain incomplete parts or puzzle-specific assumptions. Migration is not evidence that every historical solution is complete or correct. Consult native sample tests and the validation output for the particular day/part.

The migrated inventory records which parts are implemented. An unfinished part remains an explicit stub.

| Year / day | Language | Source | Implemented parts |
| --- | --- | --- | --- |
| 2020 / 1 | Go | [solution.go](../go/y20/d01/solution.go) | 1 and 2 |
| 2020 / 2 | Elixir | [d02.ex](../elixir/lib/y20/d02.ex) | 1 and 2 |
| 2021 / 1 | Scala | [Day01.scala](../scala/src/main/scala/aoc/solutions/y2021/Day01.scala) | 1 and 2 |
| 2021 / 2 | Scala | [Day02.scala](../scala/src/main/scala/aoc/solutions/y2021/Day02.scala) | 1 and 2 |
| 2021 / 3 | Scala | [Day03.scala](../scala/src/main/scala/aoc/solutions/y2021/Day03.scala) | 1 and 2 |
| 2021 / 4 | Haskell | [Day04.hs](../haskell/src/Year2021/Day04.hs) | 1 and 2 |
| 2021 / 5 | Haskell | [Day05.hs](../haskell/src/Year2021/Day05.hs) | 2 |
| 2021 / 6 | Haskell | [Day06.hs](../haskell/src/Year2021/Day06.hs) | 2 |
| 2021 / 7 | Rust | [d07.rs](../rust/years/y2021/src/d07.rs) | 2 |
| 2021 / 8 | Rust | [d08.rs](../rust/years/y2021/src/d08.rs) | 1 |
| 2021 / 9 | Haskell | [Day09.hs](../haskell/src/Year2021/Day09.hs) | 1 and 2 |
| 2022 / 1 | Rust | [d01.rs](../rust/years/y2022/src/d01.rs) | 2 |

For example, run `./aoc test 2022 1 --lang rust --part 2`. Requesting `--part both` includes the unfinished part 1 and correctly fails rather than presenting a complete suite.

Moving a real input out of tracked files preserves its local content under `data/`. Ignoring or removing it from the current tree does not remove it from existing Git history. This work does not rewrite history or change the repository's visibility.

## Migration validation

On 2026-10-15, the migration passed `./scripts/check.sh` across all five languages: 19 migrated sample assertions, 14 toolkit tests, and 10 Rust workspace tests, alongside the Go, Elixir, Cabal, and Scala checks. CLI scaffolding was checked in temporary copies for each language: registration and compilation succeeded, duplicate generation preserved existing files, and unfinished runs or empty sample suites failed as expected.

The current Scala and Elixir APIs were also checked separately: all 19 Scala harness tests and 10 Elixir tests pass. Scala coverage includes pure values, bounded parallel execution on ordinary text-based days, typed failures, and resource cleanup. Scala's packaged runner passes all six existing sample assertions and preserves all six recorded real-input answers. Scratch CLI checks cover pure Scala templates, ordinary-day parallel computations with pure callbacks, streaming discovery, UTF-8 lines, arbitrary raw bytes, missing samples, unfinished stubs, and overwrite refusal. Elixir scratch checks cover compile-time sample declarations, native Mix tasks, central input defaults, single-file scaffolding, partial-part development, assertion failures, and overwrite protection. Both languages pass root CLI run/test checks; their format checks pass.

Mocked download/cache tests passed. Live puzzle download in the onboarding environment was blocked by a proxy `CONNECT 403`; access to `adventofcode.com` needs to be enabled and retried. Authenticated live input fetching was not tested because `AOC_SESSION` was not set. Interactive editor testing is described separately in [the editor guide](editors.md#what-is-verified).

## Boundaries

Custom languages supply trusted local commands and a stub template; they still need their own toolchain, runner, tests, dependencies, and editor integration. A single `add-language` command registers them with the orchestrator, but cannot invent a language's development tools.

The common benchmark measures process execution, including native runtime overhead. It is useful for following changes in the same solution, and is not a statistical microbenchmark framework.

Puzzle/input caching works without a network connection once populated. Compiler and package caches remain the responsibility of Cargo, Go, Mix, Scala CLI, and Cabal.
