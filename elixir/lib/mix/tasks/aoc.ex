defmodule Mix.Tasks.Aoc do
  use Mix.Task

  @shortdoc "Runs a compiled puzzle part using standard input"
  @moduledoc "Run `mix compile`, then `mix aoc YEAR DAY PART < input.txt`."

  @impl Mix.Task
  def run([year, day, part]) do
    Mix.Task.run("app.start", ["--no-compile"])
    input = IO.read(:stdio, :eof)
    result = Advent.Runner.solve(integer!(year), integer!(day), integer!(part), input)
    IO.puts(result)
  rescue
    error -> Mix.raise(Exception.message(error))
  end

  def run(_args), do: Mix.raise("usage: mix aoc YEAR DAY PART (1 or 2)")

  defp integer!(text) do
    case Integer.parse(text) do
      {number, ""} -> number
      _ -> Mix.raise("invalid integer #{inspect(text)}")
    end
  end
end
