defmodule Mix.Tasks.Aoc do
  use Mix.Task

  @shortdoc "Alias for aoc.run (legacy YEAR DAY PART reads standard input)"
  @moduledoc "Use `mix aoc.run`, `mix aoc.test`, and `mix aoc.new`."

  @impl Mix.Task
  def run([year, day, part]) when part in ["1", "2", "both"] do
    Mix.Task.run("aoc.run", [year, day, "--part", part, "--stdin", "--no-compile"])
  end

  def run(args), do: Mix.Task.run("aoc.run", args)
end
