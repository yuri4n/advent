defmodule Advent.Scaffold do
  @moduledoc false

  def new!(root, year, day) when year in 2015..2099 and day in 1..25 do
    yy = year |> rem(100) |> Integer.to_string() |> String.pad_leading(2, "0")
    dd = day |> Integer.to_string() |> String.pad_leading(2, "0")
    solution_path = Path.join(root, "lib/y#{yy}/d#{dd}.ex")

    if File.exists?(solution_path) do
      raise ArgumentError, "refusing to overwrite #{solution_path}"
    end

    solution = """
    defmodule Advent.Y#{yy}.D#{dd} do
      use Advent.Day, year: #{year}, day: #{day}

      # example "example input", part1: expected_answer

      def process(input), do: Advent.Input.lines(input)
      # Define parts only once implemented; absent functions are skipped by run/0.
      # def part1(_input), do: raise("TODO: implement part 1")
      # def part2(_input), do: raise("TODO: implement part 2")
    end
    """

    File.mkdir_p!(Path.dirname(solution_path))
    File.write!(solution_path, solution, [:exclusive])
    solution_path
  end

  def new!(_root, _year, _day),
    do: raise(ArgumentError, "year must be 2015..2099 and day 1..25")
end
