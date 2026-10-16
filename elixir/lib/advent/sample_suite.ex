defmodule Advent.SampleSuite do
  @moduledoc "Generates ExUnit assertions from every module that uses `Advent.Day`."

  @doc false
  def cases(module, :declared) do
    parts =
      module.__aoc__().samples
      |> Enum.flat_map(&Map.keys(&1.expectations))
      |> Enum.uniq()
      |> Enum.map(fn key -> if key == :part1, do: 1, else: 2 end)
      |> Enum.sort()

    cases(module, if(parts == [], do: [1], else: parts))
  end

  def cases(module, parts) do
    metadata = module.__aoc__()

    for part <- parts,
        {sample, index} <- samples_for(metadata.samples, part) do
      %{
        module: module,
        year: metadata.year,
        day: metadata.day,
        part: part,
        sample: sample,
        index: index
      }
    end
  end

  defp samples_for(samples, part) do
    key = if part == 1, do: :part1, else: :part2

    matching =
      samples
      |> Enum.with_index(1)
      |> Enum.filter(fn {sample, _} -> Map.has_key?(sample.expectations, key) end)

    if matching == [], do: [{nil, 0}], else: matching
  end

  defmacro __using__(_options) do
    {modules, parts} =
      case Application.get_env(:advent, :sample_selection) do
        nil -> {Advent.Runner.days(), :declared}
        {year, day, parts} -> {[Advent.Runner.day!(year, day)], parts}
      end

    tests =
      for module <- modules, test <- cases(module, parts) do
        name = "#{test.year} day #{test.day} part #{test.part} example #{test.index}"
        key = if test.part == 1, do: :part1, else: :part2

        body =
          if test.sample == nil do
            quote do
              flunk(unquote("no examples: #{module} has no #{key} expectations"))
            end
          else
            quote do
              expected = unquote(Macro.escape(Map.fetch!(test.sample.expectations, key)))

              actual =
                Advent.Runner.answer(
                  unquote(module),
                  unquote(test.part),
                  unquote(test.sample.input)
                )

              assert actual == expected,
                     unquote("inline example at #{test.sample.file}:#{test.sample.line}")
            end
          end

        quote do
          @tag year: unquote(test.year), day: unquote(test.day), part: unquote(test.part)
          test unquote(name) do
            unquote(body)
          end
        end
      end

    quote do
      use ExUnit.Case, async: true
      unquote_splicing(tests)
    end
  end
end
