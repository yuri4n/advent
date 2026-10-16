defmodule Advent.InputTest do
  use ExUnit.Case, async: true
  alias Advent.Input

  defmodule Nested do
    use Advent.Day, year: 2026, day: 20
    def process(input), do: Advent.Input.line_chars(input)
    def part1(rows), do: Enum.reduce(rows, 0, fn row, sum -> sum + Enum.count(row) end)
  end

  test "benchmarks snapshot nested streams before advancing their outer stream" do
    assert Advent.Benchmark.run(Nested, 1, "ab\n雪\n", 2, 1).answer == 3
  end

  test "byte views decode split UTF-8 and nested lines never collect a full line" do
    text = "é雪🙂\r\n\nlast"
    chunks = for <<byte <- text>>, do: <<byte>>
    assert Enum.to_list(Input.chars(Input.new(chunks))) == String.to_charlist(text)
    assert Enum.to_list(Input.lines(Input.new(chunks))) == ["é雪🙂", "", "last"]

    source =
      Stream.repeatedly(fn ->
        send(self(), :read_chunk)
        String.duplicate("a", 4096)
      end)

    assert Input.line_chars(Input.new(source)) |> Stream.map(&Enum.take(&1, 2)) |> Enum.take(1) ==
             [~c"aa"]

    assert_received :read_chunk
    refute_received :read_chunk

    assert Input.line_chars(Input.new("abcdef\r\nxyz\n")) |> Enum.map(&Enum.take(&1, 2)) == [
             ~c"ab",
             ~c"xy"
           ]

    assert_raise ArgumentError, fn -> Input.chars(Input.new(<<255>>)) |> Enum.to_list() end
  end
end
