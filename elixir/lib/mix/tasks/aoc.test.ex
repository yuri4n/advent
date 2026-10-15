defmodule Mix.Tasks.Aoc.Test do
  use Mix.Task

  @shortdoc "Tests a day's inline samples using ExUnit"
  @moduledoc "Run `mix aoc.test YEAR DAY [--part 1|2|both]`."

  @impl Mix.Task
  def run(args) do
    {options, target} = OptionParser.parse!(args, strict: [part: :string])
    {year, day} = Advent.Command.target!(target)
    parts = Advent.Command.parts!(Keyword.get(options, :part, "both"))
    Mix.Task.run("app.start")
    Advent.Runner.day!(year, day)
    previous = Application.get_env(:advent, :sample_selection)
    Application.put_env(:advent, :sample_selection, {year, day, parts})

    try do
      Mix.Task.run("test", ["test/samples_test.exs"])
    after
      if previous do
        Application.put_env(:advent, :sample_selection, previous)
      else
        Application.delete_env(:advent, :sample_selection)
      end
    end
  end
end
