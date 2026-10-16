#!/usr/bin/env bash
set -euo pipefail

root="${AOC_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$root/elixir"

check_elixir() {
  command -v elixir >/dev/null 2>&1 && command -v mix >/dev/null 2>&1 || {
    echo "Elixir >=1.14, Mix, and a compatible Erlang/OTP are required." >&2
    return 1
  }
}

check_elixir
case "${1:-}" in
  doctor)
    elixir --version
    mix --version
    elixir -e 'unless Version.match?(System.version(), "~> 1.14"), do: System.halt(1)' || {
      echo "This Mix project requires Elixir ~>1.14." >&2
      exit 1
    }
    ;;
  build)
    mix compile >&2
    ;;
  run)
    [[ $# == 4 ]] || { echo "usage: elixir.sh run YEAR DAY PART" >&2; exit 1; }
    exec mix aoc.run "$2" "$3" --part "$4" --stdin --no-compile
    ;;
  bench)
    [[ $# == 6 ]] || { echo "usage: elixir.sh bench YEAR DAY PART ITERATIONS WARMUP" >&2; exit 1; }
    exec mix aoc.bench "$2" "$3" "$4" "$5" "$6" --no-compile
    ;;
  expect)
    [[ $# == 5 ]] || { echo "usage: elixir.sh expect YEAR DAY PART INDEX" >&2; exit 1; }
    exec mix aoc.expect "$2" "$3" "$4" "$5" --no-compile
    ;;
  example)
    [[ $# == 4 ]] || { echo "usage: elixir.sh example YEAR DAY INDEX" >&2; exit 1; }
    exec mix aoc.example "$2" "$3" "$4" --no-compile
    ;;
  test)
    [[ $# == 3 || $# == 4 ]] || { echo "usage: elixir.sh test YEAR DAY [1|2|both]" >&2; exit 1; }
    exec mix aoc.test "$2" "$3" --part "${4:-both}"
    ;;
  new)
    [[ $# == 3 ]] || { echo "usage: elixir.sh new YEAR DAY" >&2; exit 1; }
    mix compile >&2
    exec mix aoc.new "$2" "$3"
    ;;
  *) echo "usage: elixir.sh {doctor|build YEAR|run YEAR DAY PART|test YEAR DAY [PART]|example YEAR DAY INDEX|new YEAR DAY}" >&2; exit 1 ;;
esac
