#!/usr/bin/env bash
set -euo pipefail
root="${AOC_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$root/haskell"
command="${1:-}"
shift || true
case "$command" in
  doctor)
    result=0
    for tool in ghc cabal python3; do
      if command -v "$tool" >/dev/null 2>&1; then
        "$tool" --version
      else
        printf '%s is missing\n' "$tool" >&2
        result=1
      fi
    done
    exit "$result"
    ;;
  build)
    exec cabal build exe:advent-exe --offline
    ;;
  run|test)
    if [[ "$command" == test ]]; then
      cabal build exe:advent-exe --offline >&2
    fi
    binary="$(cabal list-bin exe:advent-exe --offline)"
    if [[ ! -x "$binary" ]]; then
      printf 'Haskell runner is not built; run cabal build exe:advent-exe --offline in haskell first.\n' >&2
      exit 1
    fi
    exec "$binary" "$command" "$@"
    ;;
  new)
    exec python3 tools/new_day.py "$@"
    ;;
  *)
    printf 'Usage: haskell.sh build YEAR | run YEAR DAY PART | test YEAR DAY [PART] | new YEAR DAY | doctor\n' >&2
    exit 2
    ;;
esac
