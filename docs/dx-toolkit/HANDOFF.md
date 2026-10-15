# DX toolkit: handoff

<!-- claude: created this section (2026-10-15) -->
## Status

- This document is the output of an exploration session. The session wrote no toolkit code.
- The user made no final design decisions. Section 5 lists the open decisions.
- Do not start to build before the user answers the open decisions in Section 5.
- Branch: `claude/gifted-faraday-n1q8dp`.

<!-- claude: created this section (2026-10-15) -->
## 1. Goal

Make a developer-experience (DX) toolkit for Advent of Code (AoC) in this repository.

- The toolkit must support the two parts of each puzzle day.
- The toolkit must support all the languages in the repository: Rust, Go, Elixir, Scala, Haskell.
- The user wants a "very focused" DX. The solution file must contain the solution and almost nothing else.
- The user wants compile-time features where the language has them. Example: Rust macros that read the input at compile time.
- The toolkit must not solve puzzles. Templates must only make empty stubs (`todo!()`, `undefined`, `???`).

<!-- claude: created this section (2026-10-15) -->
## 2. Current state of the repository

Each language has its own harness. The conventions are different in each language.

| Language | Input location | Read method | Sample check | Run method |
|---|---|---|---|---|
| Rust | `rust/modules/yYY/dDD/`, sometimes in `src/` | `utils::Solver` at runtime; `y22/d01` uses `include_str!` | Yes (`Solver` only) | One crate for each day |
| Go | `go/yYY/dDD/{input,example}.txt` | Runtime, with a path fallback | No | `go run` for each day |
| Elixir | `elixir/inputs/day_02_input.txt` (no year) | Runtime stream | ExUnit test | `.exs` file in `lib/` that calls itself |
| Scala | `scala/src/main/resources/dayNN_input.txt` (no year) | Class-loader resource | No | One `IOApp` for each day |
| Haskell | `haskell/inputs/year-21/day-NN/` | Standard input | No (`test/Spec.hs` is empty) | `app/Main.hs` imports one day (hard-coded) |

### Known defects (found during the exploration)

- `rust/modules/y21/d07/src/main.rs` reads `year21/day07/input.txt`. That path does not exist after the folder refactor.
- Rust sample files have different names: `sample_input.txt` (d08), `sample-input.txt` (d07, y22/d01). `utils::Solver` expects `sample_input.txt`.
- Rust package names do not agree: `year21_day07`, `y21d08`, `day01`. The name `day01` can collide with a future year.
- `mix` does not compile `.exs` files in `elixir/lib/`. `d02.exs` runs itself at the bottom of the file.
- `README.md` has bad links. The link `rust/y21/` does not exist. "Elxir" is a typo.
- The repository contains real puzzle inputs. The AoC author asks people not to publish inputs. Make sure that the repository is private, or remove the inputs (see decision D1).

<!-- claude: created this section (2026-10-15) -->
## 3. Proposed architecture

The shared part is the contract, not the code: `(year, day) -> (input, samples, expected answers) -> (part1, part2)`.

1. **Data contract.** One location for each day. Proposal: `data/yYY/dDD/{input.txt, sample.txt, answers.toml}`. `answers.toml` holds the sample answers. It can also hold the real answers, to find regressions after a refactor.
2. **Orchestrator CLI (`aoc`).** Commands:
   - `new <year> <day> --lang <lang>`: make a day from a template.
   - `fetch`: download the input with the session cookie, then keep a local copy.
   - `run` and `test`: call the correct build tool (cargo, go, mix, sbt, stack).
   - `readme`: make the `README.md` index from the file system. This removes the bad-link problem.
3. **One adapter for each language.** Each adapter embeds the data at compile time and makes `main`, sample tests and timing. The user writes only `parse`, `part1` and `part2`.
4. **Environment.** A Nix flake or a devcontainer that pins the five toolchains. Cloud sessions have no toolchains installed.
5. **CI.** A GitHub Actions matrix that runs `aoc test` for each language.

### Compile-time embedding in each language

| Language | Mechanism | Limit |
|---|---|---|
| Rust | `include_str!`; a proc-macro can also read `answers.toml` and make `#[test]` functions | `macro_rules!` cannot parse TOML. A proc-macro crate is necessary. |
| Go | `//go:embed input.txt` | The pattern cannot contain `..`. It cannot embed symbolic links. The file must be in the package directory. |
| Elixir | `@external_resource path` and `@input File.read!(path)` | None of importance. The module compiles again when the file changes. |
| Haskell | Template Haskell `embedFile` (package `file-embed`) with `makeRelativeToProject` | Template Haskell increases compile time. |
| Scala 3 | Inline macros can read files; or set `unmanagedResourceDirectories` to the data directory | The macro approach is more work than the resource approach. |

<!-- claude: created this section (2026-10-15) -->
## 4. Rust API exploration

The user gave this sketch as an idea, not as an instruction:

```rust
#[advent_day]
mod blah {
    #[day1]
    fn part1 /* ... */
}
```

### Constraint

A proc macro sees only the tokens that it is attached to.

- An attribute on an inline module (`#[aoc] mod x { ... }`) sees the full module. It can find `#[part1]` and `#[part2]`, remove them, and make `main` and tests.
- On stable Rust, an attribute macro on a non-inline module (`#[aoc] mod x;`) gives error E0658. A file-level inner attribute (`#![aoc]`) is also unstable.
- An attribute on a function (`#[part1] fn ...`) cannot see the other functions. A collector is necessary: a naming convention with an `aoc::main!()` macro, or link-time registration (`linkme` or `inventory`).

### Variants

- **A. Module attribute (the sketch).** `#[aoc(2022, 1)] mod day { #[part1] fn p1(..) .. }`. Explicit. No linker tricks. The cost is one wrapper module and one indent level in each file.
- **B. Year and day from the path.** The macro reads `CARGO_MANIFEST_DIR` or the file path (`y22/d01.rs`). The user writes only `#[part1] fn ...` and `#[part2] fn ...`. A collector is necessary. The path convention becomes part of the API.
- **C. B with a parse step.** `#[parse] fn parse(s: &str) -> T`. The macro reads the argument type of each part to choose `&str` or `&T`. Parse time shows separately. Same idea as `#[aoc_generator]` in `cargo-aoc`.
- **Samples (for all variants).** Inline in the source, for example `sample!(r"...", part1 = 24000, part2 = 45000);`. More than one `sample!` is permitted, because part 2 often has a different example. Each `sample!` becomes a `#[test]`. `run` checks the samples first and stops if one fails.

### Recommendation (not approved)

Variant B with inline samples. Variant C as an option. Use one crate for each year, with one module for each day. Run with `cargo run -- 22 1`. Test with `cargo test y22::d01`.

<!-- claude: created this section (2026-10-15) -->
## 5. Open decisions

Ask the user these questions. Do not choose for the user.

| ID | Question | Options | Recommendation |
|---|---|---|---|
| D1 | Keep real inputs in git? | (a) `.gitignore` and `aoc fetch`; (b) private submodule; (c) git-crypt | (a) |
| D2 | Central `data/` directory, or data in each language directory? | Central (Go then needs a copy step or a runtime read) / per language | Central |
| D3 | Language for the `aoc` CLI? | `just` and shell / Rust binary / Python with `uv` | Rust |
| D4 | Build tools | Keep `stack` and `sbt`, or change to `cabal` and `scala-cli` | No recommendation |
| D5 | Old solutions | Move them to the new harness, or keep them as they are | No recommendation |
| D6 | Scope | Five languages only, or "add a language" as one command | No recommendation |
| D7 | Extra features | Auto-submit, timing and benchmarks, puzzle text in the terminal | Ask which ones the user will use |
| R1 | Rust collector | `linkme` (link-time) or an explicit `aoc::main!()` | No recommendation |
| R2 | Rust parse step (variant C) | Yes / no | No recommendation |
| R3 | Samples | Inline `sample!` or `.txt` files | Inline |
| R4 | Rust crate layout | One crate for each year, or one crate for each day | One crate for each year |
| R5 | Run loop | `cargo run -- 22 1` only, or watch mode (`bacon` or `cargo-watch`) | Watch mode |
| X1 | Cross-language shape | Same API shape in all languages, or the idiomatic shape for each language with a shared data contract only | Most important open decision. Go has no macros (only `go generate`), so a uniform shape is difficult there. |

<!-- claude: created this section (2026-10-15) -->
## 6. Proposed order of work

1. Get the answers to D1, D2, D3, R1 to R5 and X1.
2. Make the data contract and the CLI (`new`, `fetch`, `test`, `readme`).
3. Make the Rust adapter. Use it as the reference implementation.
4. Make the Go, Elixir, Haskell and Scala adapters.
5. Move the old solutions (if D5 says so). Add CI and the Nix flake or devcontainer.

<!-- claude: created this section (2026-10-15) -->
## 7. User preferences for the next session

- Mark each change made by Claude at the smallest possible part (function or code fragment). Do not mark full files.
- Write documentation in ASD-STE100.
- Ask about missing details before you build. Show options and tradeoffs. Challenge the user when necessary.
