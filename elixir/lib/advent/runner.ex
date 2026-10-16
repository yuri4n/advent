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

  def solve(year, day, part, input) when part in [1, 2],
    do: answer(day!(year, day), part, input)

  def solve(_year, _day, _part, _input), do: raise(ArgumentError, "part must be 1 or 2")

  def available_parts(module),
    do: Enum.filter([1, 2], &function_exported?(module, function(&1), 1))

  def function(1), do: :part1
  def function(2), do: :part2

  def prepare(module, input) do
    if function_exported?(module, :process, 1) do
      input |> Advent.Input.new() |> module.process()
    else
      Advent.Input.text(input)
    end
  end

  def answer(module, part, input) when part in [1, 2] do
    unless part in available_parts(module),
      do: raise(ArgumentError, "part #{part} is unavailable")

    apply(module, function(part), [prepare(module, input)])
  end

  def auto(module, path \\ nil) do
    metadata = module.__aoc__()
    path = path || Advent.Command.input_path(metadata.year, metadata.day)

    for part <- available_parts(module) do
      for {example, index} <- Enum.with_index(metadata.examples, 1),
          Map.has_key?(example.expectations, function(part)) or
            map_size(example.expectations) == 0 do
        actual = answer(module, part, example.input)

        case Map.fetch(example.expectations, function(part)) do
          {:ok, expected} when actual != expected ->
            raise "example #{index} part #{part}: expected #{inspect(expected)}, got #{inspect(actual)}"

          _ ->
            IO.puts("Example #{index} part #{part}: #{actual}")
        end
      end

      if File.regular?(path),
        do: IO.puts("Input part #{part}: #{answer(module, part, Advent.Input.file(path))}")
    end

    :ok
  end

  def example_input(module, index) do
    examples = module.__aoc__().examples

    if index < 1 or index > length(examples),
      do: raise(ArgumentError, "example #{index} does not exist")

    Enum.at(examples, index - 1).input
  end
end
