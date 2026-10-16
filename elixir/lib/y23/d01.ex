defmodule Advent.Y23.D01 do
  use Advent.Day, year: 2023, day: 1

  example("1abc2\npqr3stu8vwx\na1b2c3d4e5f\ntreb7uchet\n", part1: 142)

  example(
    """
    two1nine
    eightwothree
    abcone2threexyz
    xtwone3four
    4nineeightseven2
    zoneight234
    7pqrstsixteen
    """,
    part2: 281
  )

  import NimbleParsec
  defparsec(:calibration, ascii_string([?a..?z, ?0..?9], min: 1) |> eos())

  def process(input) do
    input
    |> Advent.Input.lines()
    |> Stream.reject(&(&1 == ""))
    |> Advent.Parse.lines(&calibration/1)
    |> Stream.map(&hd/1)
  end

  @spec is_digit(char) :: boolean
  defp is_digit(char), do: char in ?0..?9

  defp fetch_numbers(line), do: line |> String.to_charlist() |> Enum.filter(&is_digit/1)

  defp build_number(num_chars) do
    [List.first(num_chars), List.last(num_chars)] |> to_string() |> Integer.parse() |> elem(0)
  end

  @spec part1(Enumerable.t()) :: number()
  def part1(input) do
    input
    |> Stream.map(&fetch_numbers/1)
    |> Stream.map(&build_number/1)
    |> Enum.reduce(&+/2)
  end

  @replacements %{
    "one" => "1",
    "two" => "2",
    "three" => "3",
    "four" => "4",
    "five" => "5",
    "six" => "6",
    "seven" => "7",
    "eight" => "8",
    "nine" => "9"
  }

  defp with_value(_, _, _, from \\ 0, res \\ [])

  defp with_value("", _, _, _, res), do: res

  defp with_value(line, str, num, from, res) do
    case :binary.match(line, str) do
      :nomatch ->
        res

      {pos, len} ->
        after_relative = pos + len + 1
        new_from = from + after_relative
        new_match = {from + pos, num}

        with_value(String.slice(line, after_relative..-1//1), str, num, new_from, [
          new_match | res
        ])
    end
  end

  defp build_and_fetch_nums(line) do
    @replacements
    |> Enum.flat_map(fn {text, num} ->
      with_value(line, text, num) ++ with_value(line, num, num)
    end)
    |> Enum.sort(fn {a, _}, {b, _} -> a <= b end)
    |> Enum.map(fn {_, x} -> x end)
  end

  @spec part2(Enumerable.t()) :: number()
  def part2(input) do
    input
    |> Stream.map(&build_and_fetch_nums/1)
    |> Stream.map(fn list -> {List.first(list), List.last(list)} end)
    |> Stream.map(fn {f, l} -> f <> l end)
    |> Stream.map(&Integer.parse/1)
    |> Stream.map(&elem(&1, 0))
    |> Enum.reduce(&+/2)
  end
end
