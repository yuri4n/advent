defmodule Advent.Command do
  @moduledoc false

  def target!([year, day]) do
    year = integer!(year)
    year = if year < 100, do: year + 2000, else: year
    day = integer!(day)

    unless year in 2015..2099 and day in 1..25 do
      Mix.raise("year must be 2015..2099 (or 15..99) and day must be 1..25")
    end

    {year, day}
  end

  def target!(_args), do: Mix.raise("expected YEAR DAY")

  def parts!("both"), do: [1, 2]
  def parts!("1"), do: [1]
  def parts!("2"), do: [2]
  def parts!(_part), do: Mix.raise("part must be 1, 2, or both")

  def input_path(year, day) do
    root =
      System.get_env("AOC_ROOT") || Path.expand("..", Path.dirname(Mix.Project.project_file()))

    yy = year |> rem(100) |> Integer.to_string() |> String.pad_leading(2, "0")
    dd = day |> Integer.to_string() |> String.pad_leading(2, "0")
    Path.join(root, "data/y#{yy}/d#{dd}/input.txt")
  end

  defp integer!(text) do
    case Integer.parse(text) do
      {number, ""} -> number
      _ -> Mix.raise("invalid integer #{inspect(text)}")
    end
  end
end
