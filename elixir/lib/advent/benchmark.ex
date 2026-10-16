defmodule Advent.Benchmark do
  @moduledoc false

  def run(module, part, input, iterations, warmup) do
    unless iterations in 1..10_000 and warmup in 0..1000,
      do: raise(ArgumentError, "invalid benchmark counts")

    unless part in Advent.Runner.available_parts(module),
      do: raise(ArgumentError, "part #{part} is unavailable")

    totals =
      for _ <- 1..(iterations + warmup),
          do: timed(fn -> Advent.Runner.answer(module, part, input) end)

    {replay, setup} =
      timed(fn ->
        prepared = Advent.Runner.prepare(module, input)

        if function_exported?(module, :snapshot, 1),
          do: module.snapshot(prepared),
          else: snapshot(prepared)
      end)

    solves =
      for _ <- 1..(iterations + warmup) do
        prepared = replay.()
        timed(fn -> apply(module, Advent.Runner.function(part), [prepared]) end)
      end

    expected = totals |> hd() |> elem(0)

    unless Enum.all?(totals ++ solves, fn {answer, _} -> answer == expected end),
      do: raise("answer changed between benchmark runs or modes")

    timings =
      Enum.zip(Enum.drop(totals, warmup), Enum.drop(solves, warmup))
      |> Enum.map(fn {{_, total}, {_, solve}} -> {total, solve} end)

    %{answer: expected, setup: setup, timings: timings}
  end

  def print(report) do
    IO.puts("AOC_BENCH_V1\n#{report.setup}")
    Enum.each(report.timings, fn {total, solve} -> IO.puts("#{total} #{solve}") end)
    IO.puts(report.answer)
  end

  defp snapshot(value)
       when is_struct(value, Stream) or is_struct(value, File.Stream) or is_function(value, 2) do
    # Snapshot inner streams before the outer stream advances its borrowed line.
    saved = Enum.map(value, &snapshot/1)
    fn -> Stream.map(Enum.map(saved, & &1.()), & &1) end
  end

  defp snapshot(value), do: fn -> value end

  defp timed(work) do
    start = System.monotonic_time(:nanosecond)
    result = work.()
    {result, System.monotonic_time(:nanosecond) - start}
  end
end
