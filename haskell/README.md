# Haskell

Open this directory in your editor. `advent.cabal` is the source of truth and
`cabal.project` lets Haskell Language Server discover the library, executable,
and tests. The [Nix and direnv setup](../docs/manual.md#set-up-the-tools) selects
the Haskell environment on entry after you approve its `.envrc` with
`direnv allow`. Alternatively, run `nix develop .#haskell` from the repository
root. With manual toolchains, install HLS built for your GHC version.
Fourmolu reads `fourmolu.yaml`.

From the repository root:

```sh
aoc new 2026 1 --lang haskell
aoc edit 2026 1 --lang haskell
aoc run 2021 9 --lang haskell --part 1
aoc test 2021 9 --lang haskell
```

The adapter requires `ghc`, `cabal`, and Python 3 (only for scaffolding). The
Nix language shell provides them, matching HLS, and the project's GHC packages.
For a fresh manual installation, run `cabal update` and `cabal build all` in this
directory once. Native commands work with the populated dependencies offline:

```sh
cabal build all --offline
cabal test --offline --test-show-details=direct
cabal run --offline advent-exe -- test 2021 9
cabal run --offline advent-exe -- run 2021 9 1 < ../data/y21/d09/input.txt
```

Each day exports pure `parse`, `part1`, `part2`, and `samples`. Change `Input`
and the result types to fit your solution. Results need a `Show` instance.
The shared harness handles input, output, selection, and assertions:

```haskell
module Year2026.Day01 (parse, part1, part2, samples) where

import AOC (Sample (..))

type Input = String

parse :: String -> Input
parse = id

part1 :: Input -> Integer
part1 = undefined

part2 :: Input -> Integer
part2 = undefined

samples :: [Sample]
samples = []
```

Add inline assertions with `Sample "sample input" (Just "expected part 1")
(Just "expected part 2")`; use `Nothing` for a part you are not asserting yet.
Expected values are the exact `show` representation of the answer. A test with
no assertions fails. Select one part with `test YEAR DAY 1` or `test YEAR DAY 2`.
The generated day contains only empty stubs and registers itself in both the
Cabal manifest and `AOC.Registry`; existing days are never overwritten.

The migrated 2021 day 5 and day 6 solutions only implemented part two. Their
part-one functions remain `undefined`. The other existing solutions retain
their original algorithms. The native suite checks six existing sample part
answers across days 4, 5, 6, and 9.
