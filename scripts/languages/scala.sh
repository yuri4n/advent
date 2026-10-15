#!/usr/bin/env bash
set -euo pipefail

root="${AOC_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
project="$root/scala"
jar="$root/.aoc/build/scala/advent.jar"
scala_cli="${SCALA_CLI:-scala-cli}"
java_command="${AOC_JAVA:-java}"

fail() { printf 'Scala: %s\n' "$*" >&2; exit 1; }
check_day() {
  [[ "${1:-}" =~ ^[0-9]{4}$ ]] && ((10#$1 >= 2015)) || fail 'year must be four digits, at least 2015'
  [[ "${2:-}" =~ ^[0-9]{1,2}$ ]] && ((10#$2 >= 1 && 10#$2 <= 25)) || fail 'day must be 1..25'
}
build() {
  command -v "$scala_cli" >/dev/null || fail 'install scala-cli, or set SCALA_CLI to its executable'
  command -v "$java_command" >/dev/null || fail 'install a JDK (17 or newer), or set AOC_JAVA'
  mkdir -p "$(dirname "$jar")"
  local offline_options=()
  [[ "${AOC_OFFLINE:-0}" == 1 ]] && offline_options+=(--offline)
  (cd "$project" && "$scala_cli" --power package . --assembly --server=false --coursier-validate-checksums \
    "${offline_options[@]}" --main-class aoc.Main --output "$jar" --force) >&2
}

case "${1:-}" in
  build)
    [[ $# == 2 && "$2" =~ ^[0-9]{4}$ ]] || fail 'usage: build YEAR'
    build
    ;;
  run)
    [[ $# == 4 ]] || fail 'usage: run YEAR DAY PART'
    check_day "$2" "$3"
    [[ "$4" == 1 || "$4" == 2 ]] || fail 'part must be 1 or 2'
    [[ -f "$jar" ]] || build
    exec "$java_command" -jar "$jar" run "$2" "$3" "$4"
    ;;
  test)
    [[ $# == 3 || $# == 4 ]] || fail 'usage: test YEAR DAY [1|2|both]'
    check_day "$2" "$3"
    build
    exec "$java_command" -jar "$jar" test "$2" "$3" "${4:-both}"
    ;;
  new)
    [[ $# == 3 ]] || fail 'usage: new YEAR DAY'
    check_day "$2" "$3"
    printf -v day '%02d' "$((10#$3))"
    directory="$project/src/main/scala/aoc/solutions/y$2"
    file="$directory/Day$day.scala"
    mkdir -p "$directory"
    # noclobber makes preserving existing solutions atomic.
    (set -o noclobber; cat > "$file" <<EOF
package aoc.solutions.y$2

import aoc.{Day, Sample}

object Day$day extends Day[Long, Long] {
  def part1(input: String): Long = ???
  def part2(input: String): Long = ???

  // Add inline samples, e.g. Sample("input", expected1 = Some("expected answer")).
  override val samples: Seq[Sample] = Seq.empty
}
EOF
    ) || fail "refusing to overwrite $file"
    printf 'Created %s\n' "$file"
    ;;
  doctor)
    command -v "$scala_cli" >/dev/null || fail 'scala-cli missing; install it or set SCALA_CLI'
    command -v "$java_command" >/dev/null || fail 'JDK missing; install Java 17+ or set AOC_JAVA'
    "$scala_cli" version
    "$java_command" -version
    ;;
  *) fail 'usage: scala.sh build YEAR | run YEAR DAY PART | test YEAR DAY [PART] | new YEAR DAY | doctor' ;;
esac
