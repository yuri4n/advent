defmodule Advent.Day do
  @moduledoc """
  Declares a puzzle day and its inline sample expectations.

      use Advent.Day, year: 2020, day: 2
      sample "example input", part1: 2, part2: 1

  Keep `part1/1` and `part2/1` as ordinary functions. Expectations may cover
  either part; testing a part with no expectations fails explicitly.
  """

  defmacro __using__(options) do
    year = Keyword.fetch!(options, :year)
    day = Keyword.fetch!(options, :day)

    unless year in 2015..2099 and day in 1..25 do
      raise ArgumentError, "Advent.Day requires year: 2015..2099 and day: 1..25"
    end

    quote do
      import Advent.Day, only: [sample: 2]
      Module.register_attribute(__MODULE__, :advent_samples, accumulate: true)
      @advent_year unquote(year)
      @advent_day unquote(day)
      @before_compile Advent.Day
    end
  end

  defmacro sample(input, expectations) do
    quote do
      @advent_samples Advent.Day.sample!(
                        unquote(input),
                        unquote(expectations),
                        __ENV__.file,
                        __ENV__.line
                      )
    end
  end

  @doc false
  def sample!(input, expectations, file, line) do
    unless is_binary(input) and Keyword.keyword?(expectations) and expectations != [] and
             Enum.all?(Keyword.keys(expectations), &(&1 in [:part1, :part2])) do
      raise ArgumentError,
            "sample expects an input string and part1: and/or part2: answers at #{file}:#{line}"
    end

    if length(Keyword.keys(expectations)) != length(Enum.uniq(Keyword.keys(expectations))) do
      raise ArgumentError, "duplicate sample expectation at #{file}:#{line}"
    end

    %{input: input, expectations: Map.new(expectations), file: file, line: line}
  end

  defmacro __before_compile__(env) do
    metadata = %{
      year: Module.get_attribute(env.module, :advent_year),
      day: Module.get_attribute(env.module, :advent_day),
      samples: env.module |> Module.get_attribute(:advent_samples) |> Enum.reverse()
    }

    quote do
      @doc false
      def __aoc__, do: unquote(Macro.escape(metadata))
    end
  end
end
