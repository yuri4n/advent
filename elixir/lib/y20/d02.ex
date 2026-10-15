defmodule Advent.Y20.D02 do
  def part1(input) do
    input
    |> parse()
    |> Enum.count(&pwd_is_valid?/1)
  end

  def part2(input) do
    input
    |> parse()
    |> Enum.count(&pwd_is_valid_sp?/1)
  end

  defp parse(input) do
    input
    |> String.split("\n", trim: true)
    |> Enum.map(&parse_line/1)
  end

  defp parse_line(line) do
    [str_range, str_letter, pwd] = String.split(line)
    [first, last] = String.split(str_range, "-")
    range = Range.new(String.to_integer(first), String.to_integer(last))
    {String.trim_trailing(str_letter, ":"), range, pwd}
  end

  defp pwd_is_valid?({letter, range, pwd}) do
    count = Enum.count(String.graphemes(pwd), &(letter == &1))
    count in range
  end

  defp pwd_is_valid_sp?({letter, first..last//_, pwd}) do
    fp = String.at(pwd, first - 1) == letter
    lp = String.at(pwd, last - 1) == letter
    (fp and not lp) or (not fp and lp)
  end
end
