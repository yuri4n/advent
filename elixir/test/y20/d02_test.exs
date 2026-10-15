defmodule Advent.Y20.D02Test do
  use ExUnit.Case, async: true

  @sample """
  1-3 a: abcde
  1-3 b: cdefg
  2-9 c: ccccccccc
  """

  @tag :part1
  test "part 1 sample" do
    assert Advent.Y20.D02.part1(@sample) == 2
  end

  @tag :part2
  test "part 2 sample" do
    assert Advent.Y20.D02.part2(@sample) == 1
  end
end
