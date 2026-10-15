# Elixir puzzles

Open this directory as a Mix project. Elixir 1.14 or later and a compatible
Erlang/OTP are required; there are no external package dependencies.

With the [Nix and direnv setup](../docs/manual.md#set-up-the-tools), entering this
directory selects the Elixir environment after you approve its `.envrc` with
`direnv allow`. Alternatively, run `nix develop .#elixir` from the repository
root. `aoc edit 2020 2 --lang elixir` opens the solution with the project tools
and ElixirLS available.

A day declares its metadata and samples once, then uses ordinary functions:

```elixir
defmodule Advent.Y20.D02 do
  use Advent.Day, year: 2020, day: 2

  sample """
         1-3 a: abcde
         1-3 b: cdefg
         2-9 c: ccccccccc
         """,
    part1: 2,
    part2: 1

  def part1(input), do: raise("TODO: implement part 1")
  def part2(input), do: raise("TODO: implement part 2")
end
```

`use Advent.Day` registers metadata at compile time. `sample` records inline input
and expected answers; the shared ExUnit suite generates the assertions. You can
add several samples, give a sample only `part1:` or `part2:`, and keep parsing in
an ordinary private helper. A day needs no runner, file reads, or companion test
module. Templates contain empty solution stubs only.

From this directory:

```sh
mix aoc.new 2026 1
mix aoc.test 2020 2 --part 1
mix aoc.run 2020 2 --part 1
mix aoc.run 2020 2 --input /path/to/input.txt
mix test
mix format
```

`aoc.run` reads `../data/yYY/dDD/input.txt` by default. `--input FILE` overrides it;
`--stdin` reads standard input. `--part` accepts `1`, `2`, or `both` (the default).
A single selected part prints its answer alone. Years may be `2020` or `20`.

`mix test` checks the expectations you have written, allowing work on part 1 while
part 2 remains a stub. A completely empty day fails with a missing-sample message.
`mix aoc.test YEAR DAY --part PART` requires expectations for every selected part,
so selecting `both` fails when only part 1 has samples. Failed answers and missing
expectations return nonzero status. New days are discovered automatically, and
`aoc.new` refuses to overwrite an existing module.

The repository `aoc` command uses these same Mix tasks. Its adapter builds
before running, then uses `mix aoc.run --stdin --no-compile` so answer output stays
clean and benchmark loops exclude compilation. Elixir runtime startup remains in
the reported process timings.

See [the shared manual](../docs/manual.md) for input downloads, watch mode,
benchmarks, and editor setup. ElixirLS uses this Mix project without per-day
configuration.
