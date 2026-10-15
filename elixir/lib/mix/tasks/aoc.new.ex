defmodule Mix.Tasks.Aoc.New do
  use Mix.Task

  @shortdoc "Creates empty solution and inline sample stubs without overwriting files"

  @impl Mix.Task
  def run([year, day]) do
    path = Advent.Scaffold.new!(File.cwd!(), String.to_integer(year), String.to_integer(day))
    Mix.shell().info("Created #{path}")
  rescue
    error -> Mix.raise(Exception.message(error))
  end

  def run(_args), do: Mix.raise("usage: mix aoc.new YEAR DAY")
end
