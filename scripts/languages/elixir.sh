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

date_path() {
  [[ "$1" =~ ^20[0-9]{2}$ && "$2" =~ ^[0-9]{1,2}$ ]] || { echo "invalid year/day" >&2; return 1; }
  local year=$((10#$1)) day=$((10#$2))
  (( year >= 2015 && day >= 1 && day <= 25 )) || { echo "invalid year/day" >&2; return 1; }
  printf 'test/y%02d/d%02d_test.exs' "$((year % 100))" "$day"
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
    exec mix aoc "$2" "$3" "$4"
    ;;
  test)
    [[ $# == 3 || $# == 4 ]] || { echo "usage: elixir.sh test YEAR DAY [1|2|both]" >&2; exit 1; }
    path=$(date_path "$2" "$3")
    [[ -f "$path" ]] || { echo "No Elixir samples found at $path." >&2; exit 1; }
    case "${4:-both}" in
      1|2) exec mix test "$path" --only "part$4" ;;
      both)
        mix test "$path" --only part1
        exec mix test "$path" --only part2
        ;;
      *) echo "part must be 1, 2, or both" >&2; exit 1 ;;
    esac
    ;;
  new)
    [[ $# == 3 ]] || { echo "usage: elixir.sh new YEAR DAY" >&2; exit 1; }
    mix compile >&2
    exec mix aoc.new "$2" "$3"
    ;;
  *) echo "usage: elixir.sh {doctor|build YEAR|run YEAR DAY PART|test YEAR DAY [PART]|new YEAR DAY}" >&2; exit 1 ;;
esac
