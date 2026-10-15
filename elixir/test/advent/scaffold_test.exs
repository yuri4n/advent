defmodule Advent.ScaffoldTest do
  use ExUnit.Case, async: true

  setup do
    root = Path.join(System.tmp_dir!(), "advent-scaffold-#{System.unique_integer([:positive])}")
    File.mkdir_p!(root)
    on_exit(fn -> File.rm_rf!(root) end)
    %{root: root}
  end

  test "creates native Mix modules and sample stubs and refuses replacement", %{root: root} do
    path = Advent.Scaffold.new!(root, 2026, 1)
    source = File.read!(path)
    assert source =~ "defmodule Advent.Y26.D01"
    assert source =~ ~s|raise("TODO: implement part 1")|
    assert source =~ "use Advent.Day, year: 2026, day: 1"
    refute File.exists?(Path.join(root, "test/y26/d01_test.exs"))
    assert {:ok, _} = Code.string_to_quoted(source)

    assert_raise ArgumentError, ~r/refusing to overwrite/, fn ->
      Advent.Scaffold.new!(root, 2026, 1)
    end

    assert File.read!(path) == source
  end

  test "invalid dates cannot escape the project directories", %{root: root} do
    assert_raise ArgumentError, fn -> Advent.Scaffold.new!(root, 2026, 0) end
    assert_raise ArgumentError, fn -> Advent.Scaffold.new!(root, 2000, 1) end
    assert File.ls!(root) == []
  end
end
