defmodule Advent.Runner do
  @moduledoc false

  def days do
    (:advent |> Application.spec(:modules) || [])
    |> Enum.filter(&(Code.ensure_loaded?(&1) and function_exported?(&1, :__aoc__, 0)))
    |> Enum.sort_by(fn module -> {module.__aoc__().year, module.__aoc__().day} end)
  end

  def day!(year, day) do
    case Enum.filter(days(), &(&1.__aoc__().year == year and &1.__aoc__().day == day)) do
      [module] -> module
      [] -> raise ArgumentError, "no Elixir solution for #{year} day #{day}"
      _ -> raise ArgumentError, "multiple Elixir modules declare #{year} day #{day}"
    end
  end

  def solve(year, day, part, input) when part in [1, 2] do
    function = if part == 1, do: :part1, else: :part2
    apply(day!(year, day), function, [input])
  end

  def solve(_year, _day, _part, _input), do: raise(ArgumentError, "part must be 1 or 2")
end
