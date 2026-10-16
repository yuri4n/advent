defmodule Mix.Tasks.Aoc.Example do
  use Mix.Task
  @shortdoc "Prints an inline example's input without running assertions"

  @impl Mix.Task
  def run(args) do
    {options, target} = OptionParser.parse!(args, strict: [compile: :boolean])

    case target do
      [year, day, index] ->
        {year, day} = Advent.Command.target!([year, day])
        index = String.to_integer(index)

        Mix.Task.run(
          "app.start",
          if(Keyword.get(options, :compile, true), do: [], else: ["--no-compile"])
        )

        module = Advent.Runner.day!(year, day)
        IO.write(Advent.Runner.example_input(module, index))

      _ ->
        Mix.raise("expected YEAR DAY INDEX")
    end
  rescue
    error -> Mix.raise(Exception.message(error))
  end
end
