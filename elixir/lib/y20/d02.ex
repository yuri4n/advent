defmodule Advent.Y20.D02 do
  use Advent.Day, year: 2020, day: 2

  example(
    """
    1-3 a: abcde
    1-3 b: cdefg
    2-9 c: ccccccccc
    """,
    part1: 2,
    part2: 1
  )

  import NimbleParsec

  defparsec(
    :policy,
    integer(min: 1)
    |> ignore(string("-"))
    |> integer(min: 1)
    |> ignore(string(" "))
    |> ascii_string([?a..?z], 1)
    |> ignore(string(": "))
    |> ascii_string([?a..?z], min: 1)
    |> eos()
  )

  def process(input) do
    input
    |> Advent.Input.lines()
    |> Stream.reject(&(&1 == ""))
    |> Advent.Parse.lines(&policy/1)
    |> Stream.map(fn [first, last, letter, password] ->
      {letter, Range.new(first, last), password}
    end)
  end

  def part1(input), do: Enum.count(input, &pwd_is_valid?/1)
  def part2(input), do: Enum.count(input, &pwd_is_valid_sp?/1)

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
