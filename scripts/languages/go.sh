#!/usr/bin/env bash
set -euo pipefail

root="${AOC_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$root/go"

check_go() {
  local version
  command -v go >/dev/null 2>&1 && version=$(go version) || {
    echo "Go >=1.19 is required (https://go.dev/dl/)." >&2
    return 1
  }
  if [[ ! "$version" =~ go([0-9]+)\.([0-9]+) ]] ||
      (( BASH_REMATCH[1] < 1 || (BASH_REMATCH[1] == 1 && BASH_REMATCH[2] < 19) )); then
    echo "Go >=1.19 is required; detected: $version" >&2
    return 1
  fi
}

date_path() {
  [[ "$1" =~ ^20[0-9]{2}$ && "$2" =~ ^[0-9]{1,2}$ ]] || { echo "invalid year/day" >&2; return 1; }
  local year=$((10#$1)) day=$((10#$2))
  (( year >= 2015 && day >= 1 && day <= 25 )) || { echo "invalid year/day" >&2; return 1; }
  printf './y%02d/d%02d' "$((year % 100))" "$day"
}

case "${1:-}" in
  doctor)
    check_go
    go version
    ;;
  build)
    check_go
    mkdir -p "$root/.aoc/bin"
    go build -o "$root/.aoc/bin/go-aoc" ./cmd/aoc
    ;;
  run)
    [[ $# == 4 ]] || { echo "usage: go.sh run YEAR DAY PART" >&2; exit 1; }
    [[ -x "$root/.aoc/bin/go-aoc" ]] || { echo "Run the Go build adapter first." >&2; exit 1; }
    exec "$root/.aoc/bin/go-aoc" "$2" "$3" "$4"
    ;;
  bench)
    [[ $# == 6 ]] || { echo "usage: go.sh bench YEAR DAY PART ITERATIONS WARMUP" >&2; exit 1; }
    exec "$root/.aoc/bin/go-aoc" bench "$2" "$3" "$4" "$5" "$6"
    ;;
  expect)
    [[ $# == 5 ]] || { echo "usage: go.sh expect YEAR DAY PART INDEX" >&2; exit 1; }
    exec "$root/.aoc/bin/go-aoc" expect "$2" "$3" "$4" "$5"
    ;;
  example)
    [[ $# == 4 ]] || { echo "usage: go.sh example YEAR DAY INDEX" >&2; exit 1; }
    exec "$root/.aoc/bin/go-aoc" example "$2" "$3" "$4"
    ;;
  test)
    check_go
    [[ $# == 3 || $# == 4 ]] || { echo "usage: go.sh test YEAR DAY [1|2|both]" >&2; exit 1; }
    package=$(date_path "$2" "$3")
    part="${4:-both}"
    case "$part" in
      1|2) pattern="^Test(Examples|Samples)$/^part${part}$"; expected=("part${part}") ;;
      both) pattern='^Test(Examples|Samples)$'; expected=(part1 part2) ;;
      *) echo "part must be 1, 2, or both" >&2; exit 1 ;;
    esac
    if ! output=$(go test -count=1 -v -run "$pattern" "$package" 2>&1); then
      printf '%s\n' "$output"
      exit 1
    fi
    printf '%s\n' "$output"
    for sample_part in "${expected[@]}"; do
      if ! printf '%s\n' "$output" | grep -Eq "^[[:space:]]*--- PASS: Test(Examples|Samples)/${sample_part} \\("; then
        echo "No completed Go examples found: define TestExamples/${sample_part}." >&2
        exit 1
      fi
    done
    ;;
  new)
    check_go
    [[ $# == 3 ]] || { echo "usage: go.sh new YEAR DAY" >&2; exit 1; }
    exec go run ./cmd/scaffold "$2" "$3"
    ;;
  *) echo "usage: go.sh {doctor|build YEAR|run YEAR DAY PART|test YEAR DAY [PART]|example YEAR DAY INDEX|new YEAR DAY}" >&2; exit 1 ;;
esac
