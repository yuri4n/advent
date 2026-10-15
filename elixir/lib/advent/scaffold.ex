defmodule Advent.Scaffold do
  @moduledoc false

  def new!(root, year, day) when year in 2015..2099 and day in 1..25 do
    yy = year |> rem(100) |> Integer.to_string() |> String.pad_leading(2, "0")
    dd = day |> Integer.to_string() |> String.pad_leading(2, "0")
    solution_path = Path.join(root, "lib/y#{yy}/d#{dd}.ex")
    samples_path = Path.join(root, "test/y#{yy}/d#{dd}_test.exs")

    for path <- [solution_path, samples_path] do
      if File.exists?(path), do: raise(ArgumentError, "refusing to overwrite #{path}")
      File.mkdir_p!(Path.dirname(path))
    end

    solution = """
    defmodule Advent.Y#{yy}.D#{dd} do
      def part1(_input), do: raise("TODO: implement part 1")
      def part2(_input), do: raise("TODO: implement part 2")
    end
    """

    samples = """
    defmodule Advent.Y#{yy}.D#{dd}Test do
      use ExUnit.Case, async: true

      # Keep sample input as an inline string in this file.
      @tag :part1
      test "part 1 samples" do
        flunk("no samples: add an inline input and expected part1 answer")
      end

      @tag :part2
      test "part 2 samples" do
        flunk("no samples: add an inline input and expected part2 answer")
      end
    end
    """

    File.write!(solution_path, solution, [:exclusive])

    try do
      File.write!(samples_path, samples, [:exclusive])
    rescue
      error ->
        File.rm!(solution_path)
        reraise error, __STACKTRACE__
    end

    solution_path
  end

  def new!(_root, _year, _day),
    do: raise(ArgumentError, "year must be 2015..2099 and day 1..25")
end
