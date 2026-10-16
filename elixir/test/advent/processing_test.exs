defmodule Advent.ProcessingTest do
  use ExUnit.Case, async: true

  defmodule Numbers do
    use Advent.Day, year: 2026, day: 22
    import NimbleParsec
    defparsec(:number, integer(min: 1) |> eos())
    example("7\n3\n", part1: 7, part2: 10)
    example("  unchanged\n\n")

    def process(input),
      do: Advent.Parse.lines(Advent.Input.lines(input), &number/1) |> Stream.map(&hd/1)

    def part1(numbers), do: numbers |> Enum.take(1) |> hd()
    def part2(numbers), do: Enum.sum(numbers)
  end

  defmodule PartTwo do
    use Advent.Day, year: 2026, day: 21
    def process(input), do: Advent.Input.lines(input)
    def part2(lines), do: Enum.count(lines)
  end

  test "benchmarking materializes once and automatic runs skip missing parts and inputs" do
    report = Advent.Benchmark.run(Numbers, 2, "7\n3\n", 2, 1)
    assert report.answer == 10
    assert length(report.timings) == 2
    assert Advent.Runner.available_parts(PartTwo) == [2]

    assert ExUnit.CaptureIO.capture_io(fn ->
             Advent.Runner.auto(PartTwo, "/no/such/aoc-input.txt")
           end) == ""
  end

  test "shared parsing is lazy, checks whole records, and examples use the same path" do
    assert Advent.Runner.answer(Numbers, 1, "7\nbad\n") == 7
    for _ <- 1..2, do: assert(Advent.Runner.answer(Numbers, 2, "7\r\n3\n") == 10)
    assert Advent.Runner.example_input(Numbers, 2) == "  unchanged\n\n"

    assert_raise ArgumentError, ~r/line 2/, fn ->
      Advent.Runner.answer(Numbers, 2, "7\n3junk\n")
    end
  end
end
