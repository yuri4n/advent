defmodule Advent.RunnerTest do
  use ExUnit.Case, async: true

  test "selects the requested compiled day and part" do
    input = "1-3 a: abcde\n1-3 b: cdefg\n2-9 c: ccccccccc\n"
    assert Advent.Runner.solve(2020, 2, 1, input) == 2
    assert Advent.Runner.solve(2020, 2, 2, input) == 1
  end

  test "unknown days and parts fail explicitly" do
    assert_raise ArgumentError, ~r/no Elixir solution/, fn ->
      Advent.Runner.solve(2026, 25, 1, "")
    end

    assert_raise ArgumentError, ~r/part must be/, fn ->
      Advent.Runner.solve(2020, 2, 3, "")
    end
  end
end
