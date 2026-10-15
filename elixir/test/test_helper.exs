ExUnit.start()

ExUnit.after_suite(fn stats ->
  if stats.total <= stats.excluded + stats.skipped do
    IO.puts(:stderr, "No completed Elixir samples matched the requested day and part.")
    System.halt(1)
  end
end)
