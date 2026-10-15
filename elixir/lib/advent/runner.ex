defmodule Advent.Runner do
  @moduledoc false

  def solve(year, day, part, input) when part in [1, 2] do
    name = "Elixir.Advent.Y#{pad(rem(year, 100))}.D#{pad(day)}"

    module =
      :advent
      |> Application.spec(:modules)
      |> Kernel.||([])
      |> Enum.find(&(Atom.to_string(&1) == name))

    if is_nil(module) do
      raise ArgumentError, "no Elixir solution for #{year} day #{day}"
    end

    function = if part == 1, do: :part1, else: :part2
    apply(module, function, [input])
  end

  def solve(_year, _day, _part, _input), do: raise(ArgumentError, "part must be 1 or 2")

  defp pad(number), do: number |> Integer.to_string() |> String.pad_leading(2, "0")
end
