defmodule Mix.Tasks.Aoc.Bench do
  use Mix.Task
  @shortdoc "Native with/without-processing benchmark protocol (used by aoc bench)"

  @impl Mix.Task
  def run(args) do
    {options, target} = OptionParser.parse!(args, strict: [compile: :boolean])

    case target do
      [year, day, part, iterations, warmup] ->
        {year, day} = Advent.Command.target!([year, day])
        [part] = Advent.Command.parts!(part)

        Mix.Task.run(
          "app.start",
          if(Keyword.get(options, :compile, true), do: [], else: ["--no-compile"])
        )

        input =
          case IO.binread(:stdio, :eof) do
            text when is_binary(text) -> text
            :eof -> ""
            {:error, reason} -> Mix.raise("cannot read input: #{inspect(reason)}")
          end

        Advent.Benchmark.run(
          Advent.Runner.day!(year, day),
          part,
          input,
          String.to_integer(iterations),
          String.to_integer(warmup)
        )
        |> Advent.Benchmark.print()

      _ ->
        Mix.raise("expected YEAR DAY PART ITERATIONS WARMUP")
    end
  rescue
    error -> Mix.raise(Exception.message(error))
  end
end
