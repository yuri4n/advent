defmodule Mix.Tasks.Aoc.Run do
  use Mix.Task

  @shortdoc "Runs a puzzle using its central input (or --input FILE / --stdin)"
  @moduledoc """
  Run `mix aoc.run YEAR DAY [--part 1|2|both]` from the Elixir project.
  Input defaults to `../data/yYY/dDD/input.txt`. Use `--input FILE` or `--stdin`
  to override it. `--no-compile` uses an already built project.
  """

  @impl Mix.Task
  def run(args) do
    {options, target} =
      OptionParser.parse!(args,
        strict: [part: :string, input: :string, stdin: :boolean, compile: :boolean]
      )

    {year, day} = Advent.Command.target!(target)
    parts = Advent.Command.parts!(Keyword.get(options, :part, "both"))

    if options[:stdin] && options[:input], do: Mix.raise("choose --stdin or --input")
    compile_options = if Keyword.get(options, :compile, true), do: [], else: ["--no-compile"]
    Mix.Task.run("app.start", compile_options)

    run = fn input ->
      for part <- parts do
        result = Advent.Runner.solve(year, day, part, input)
        if length(parts) == 1, do: IO.puts(result), else: IO.puts("Part #{part}: #{result}")
      end
    end

    if options[:stdin] do
      Advent.Input.with_stdin(length(parts) > 1, run)
    else
      run.(Advent.Input.file(options[:input] || Advent.Command.input_path(year, day)))
    end
  rescue
    error -> Mix.raise(Exception.message(error))
  end
end
