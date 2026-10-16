# Elixir solutions

Use the shared toolkit from the repository root, or the native Mix tasks here.

```sh
mix deps.get
mix test
mix aoc.test 2020 2 --part 1
mix aoc.run 2020 2 --part 1
mix run -e 'Advent.Y20.D02.run()'
```

A solution uses `Advent.Day`, defines a shared `process/1` over byte-backed `Advent.Input`, and implements whichever parts are ready. `process/1` may return a stream or an ordinary immutable value. Input files, stdin, and inline examples all use that same processing path.

Use **NimbleParsec directly** for grammars. `Advent.Parse.lines/2` is a small lazy bridge that attaches line context and rejects trailing input. It is not a replacement parser API.

```elixir
use Advent.Day, year: 2026, day: 1
import NimbleParsec

defparsec :number, integer(min: 1) |> eos()
def process(input), do: Advent.Parse.lines(Advent.Input.lines(input), &number/1) |> Stream.map(&hd/1)
def part1(numbers), do: Enum.sum(numbers)
example "2\n3\n", part1: 5
```

Examples store native expected values. Omit an expectation while it is unknown, or use `example "input"` to declare runnable input without an assertion. The shared ExUnit suite uses the runner, including preprocessing. `sample` and old modules without `process/1` remain supported for compatibility.

Omit an unimplemented part function rather than define one that raises. `run/0` executes available examples and existing personal input, skipping absent data/parts. Actual exceptions remain failures. Native `mix aoc.run` accepts `--stdin` or `--input PATH`; the default is central `data/yYY/dDD/input.txt`.

The root `aoc bench` reports native timings with and without processing. Streams, including nested lazy character lines, are materialized only for solve-only benchmarks, then replayed; ordinary immutable values are reused. `aoc bench --wall` and `aoc run --time` include BEAM startup in process wall time.

See [the Elixir solution guide](../docs/solutions.md#elixir), [developer manual](../docs/manual.md), and [editor setup](../docs/editors.md). The Elixir Nix shell provides compatible OTP, Elixir, and ElixirLS.
