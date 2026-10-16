defmodule Mix.Tasks.Aoc.Expect do
  use Mix.Task
  @shortdoc "Prints a selected example expectation for the shared CLI"

  @impl Mix.Task
  def run(args) do
    {options, target} = OptionParser.parse!(args, strict: [compile: :boolean])

    case target do
      [year, day, part, index] ->
        {year, day} = Advent.Command.target!([year, day])
        [part] = Advent.Command.parts!(part)
        index = String.to_integer(index)

        Mix.Task.run(
          "app.start",
          if(Keyword.get(options, :compile, true), do: [], else: ["--no-compile"])
        )

        module = Advent.Runner.day!(year, day)
        Advent.Runner.example_input(module, index)
        example = Enum.at(module.__aoc__().examples, index - 1)

        case Map.fetch(example.expectations, Advent.Runner.function(part)) do
          {:ok, value} -> IO.puts("AOC_EXPECT_V1\n#{value}")
          :error -> IO.puts("AOC_NO_EXPECT_V1")
        end

      _ ->
        Mix.raise("expected YEAR DAY PART INDEX")
    end
  rescue
    error -> Mix.raise(Exception.message(error))
  end
end
