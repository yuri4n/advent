#!/usr/bin/env bash
# Verify the toolkit and all existing inline examples without personal AoC input.
set -euo pipefail
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export AOC_ROOT="$root"
cd "$root"
cargo test --locked --manifest-path toolkit/Cargo.toml
cargo test --locked --offline --workspace --manifest-path rust/Cargo.toml
(cd go && go test -count=1 ./...)
(cd elixir && mix test)
(cd haskell && cabal test --offline --test-show-details=direct)
# Avoid empty arrays under macOS Bash 3.2 with `set -u`.
scala_test_option=
[[ "${AOC_OFFLINE:-0}" == 1 ]] && scala_test_option=--offline
(cd scala && "${SCALA_CLI:-scala-cli}" --power test . --server=false --coursier-validate-checksums ${scala_test_option:+"$scala_test_option"})
bash scripts/languages/scala.sh build 2021
for day in 1 2 3; do
  "${AOC_JAVA:-java}" -jar "$root/.aoc/build/scala/advent.jar" test 2021 "$day" both
done
"${AOC_JAVA:-java}" -jar "$root/.aoc/build/scala/advent.jar" test 2024 1 both
"${AOC_JAVA:-java}" -jar "$root/.aoc/build/scala/advent.jar" test 2025 1 both
printf 'Toolkit and all existing inline examples passed.\n'
