# Haskell solutions

This is a Cabal project. Nix supplies GHC with `split` and `megaparsec`, Cabal, and HLS; Stack and hpack are not part of the active workflow.

```sh
cabal build all --offline
cabal test --offline --test-show-details=direct
cabal run advent-exe -- test 2021 6 2
cabal run advent-exe -- run 2021 6 2 < ../data/y21/d06/input.txt
runghc -isrc src/Year2021/Day06.hs
```

Each day exports `process`, implemented part functions, typed `examples`, a registered `puzzle`, and `main`. The shared processor receives a byte-backed `AOC.Input.Input`. It can select lazy bytes, UTF-8 characters, collected string lines, or nested lazy character lines. It may return a lazy list or build an explicit graph/grid/other value. Both parts reuse the processing code and receive fresh input traversals.

Use **Megaparsec directly**. `AOC.Parsing` contains only the `Parser` type alias and complete-record runners with line context. It does not define a second combinator library.

The `solution` constructor accepts `Just part1` / `Just part2` for implemented parts and `Nothing` for unavailable ones. `Example1`, `Example2`, `ExampleBoth`, and `ExampleInput` associate typed expected answers with their parts. The registry uses each module's `puzzle`.

Run a day's `main` in the IDE or via `runghc`: it executes available examples and existing central input, skips missing data/parts, and never downloads input. Use `aoc new YEAR DAY --lang haskell` to generate an unimplemented module and update Cabal/registry entries without overwriting files.

Native benchmarks measure with and without processing. `NFData` forces the complete prepared input before solve-only timing; immutable values are then reused safely. Custom records can derive `Generic` and declare an `NFData` instance. Ordinary runs retain lazy input. `--wall` remains available for process-level timing.

Existing algorithms and unfinished parts are preserved. See [the Haskell solution guide](../docs/solutions.md#haskell), [developer manual](../docs/manual.md), and [editor setup](../docs/editors.md).
