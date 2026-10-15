defmodule Advent.Y23.D01Test do
  use ExUnit.Case, async: true

  test "the shared runner executes saved 2023 day 1 part 1" do
    input = "1abc2\npqr3stu8vwx\na1b2c3d4e5f\ntreb7uchet\n"
    assert Advent.Runner.solve(2023, 1, 1, input) == 142
  end

  test "the shared runner executes saved 2023 day 1 part 2" do
    input =
      "two1nine\neightwothree\nabcone2threexyz\nxtwone3four\n4nineeightseven2\nzoneight234\n7pqrstsixteen\n"

    assert Advent.Runner.solve(2023, 1, 2, input) == 281
  end
end
