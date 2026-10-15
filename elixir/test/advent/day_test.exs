defmodule Advent.DayTest do
  use ExUnit.Case, async: true

  defmodule PartialDay do
    use Advent.Day, year: 2026, day: 24
    sample "first", part1: 5
    sample "second", part1: 6, part2: 7
    def part1(input), do: String.length(input)
    def part2(_input), do: 7
  end

  defmodule EmptyDay do
    use Advent.Day, year: 2026, day: 25
    def part1(_input), do: raise("TODO")
    def part2(_input), do: raise("TODO")
  end

  defmodule PartOneDay do
    use Advent.Day, year: 2026, day: 23
    sample "x", part1: 1
    def part1(input), do: String.length(input)
    def part2(_input), do: raise("TODO")
  end

  test "sample declarations preserve declaration order, expected values, and locations" do
    metadata = PartialDay.__aoc__()
    assert {metadata.year, metadata.day} == {2026, 24}
    assert Enum.map(metadata.samples, & &1.input) == ["first", "second"]
    assert hd(metadata.samples).expectations == %{part1: 5}
    assert hd(metadata.samples).file == __ENV__.file
    assert hd(metadata.samples).line > 0
  end

  test "each part gets only its declared expectations" do
    assert length(Advent.SampleSuite.cases(PartialDay, [1])) == 2
    assert length(Advent.SampleSuite.cases(PartialDay, [2])) == 1
    assert Enum.all?(Advent.SampleSuite.cases(EmptyDay, [1, 2]), &is_nil(&1.sample))
  end

  test "plain mix test checks supplied expectations while selected parts require coverage" do
    assert [%{part: 1, sample: %{input: "x"}}] = Advent.SampleSuite.cases(PartOneDay, :declared)
    assert [%{part: 2, sample: nil}] = Advent.SampleSuite.cases(PartOneDay, [2])
    assert [%{sample: nil}] = Advent.SampleSuite.cases(EmptyDay, :declared)
  end

  test "invalid declarations fail at compile time" do
    assert_raise ArgumentError, ~r/year/, fn ->
      Code.compile_string("""
      defmodule InvalidAdventDate do
        use Advent.Day, year: 2026, day: 0
      end
      """)
    end

    assert_raise ArgumentError, ~r/sample expects/, fn ->
      Code.compile_string("""
      defmodule InvalidAdventSample do
        use Advent.Day, year: 2026, day: 1
        sample "input", typo: 3
      end
      """)
    end
  end
end
