defmodule Mix.Tasks.Aoc.New do
  use Mix.Task

  @shortdoc "Creates empty solution and inline sample stubs without overwriting files"

  @impl Mix.Task
  def run(args) do
    {year, day} = Advent.Command.target!(args)
    path = Advent.Scaffold.new!(File.cwd!(), year, day)
    Mix.shell().info("Created #{path}")
  rescue
    error -> Mix.raise(Exception.message(error))
  end
end
