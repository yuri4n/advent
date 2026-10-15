#!/usr/bin/env bash
# Verify the toolkit and all existing inline samples without personal AoC input.
set -euo pipefail
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export AOC_ROOT="$root"
cd "$root"
cargo test --locked --manifest-path toolkit/Cargo.toml
cargo test --locked --offline --workspace --manifest-path rust/Cargo.toml
(cd go && go test -count=1 ./...)
(cd elixir && mix test)
(cd haskell && cabal test --offline --test-show-details=direct)
scala_test_options=()
[[ "${AOC_OFFLINE:-0}" == 1 ]] && scala_test_options+=(--offline)
(cd scala && "${SCALA_CLI:-scala-cli}" --power test . --server=false --coursier-validate-checksums "${scala_test_options[@]}")
for day in 1 2 3; do
  bash scripts/languages/scala.sh test 2021 "$day" both
done
printf 'Toolkit and all existing inline samples passed.\n'
