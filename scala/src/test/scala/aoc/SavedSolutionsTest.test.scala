package aoc

import kyo.*

class SavedSolutionsTest extends PuzzleTest {
  private def run[A](value: => A < Puzzle.Effects): Result[Throwable, A] = {
    import AllowUnsafe.embrace.danger
    KyoApp.Unsafe.runAndBlock(5.seconds)(Abort.catching[Throwable](value))
  }

  test(
    "2025 day 1 checks both parts, full rotations, zero arrivals and departures"
  ) {
    val day = aoc.solutions.y2025.Day01
    assertEquals(run(Runner.checkExamples(day, "both")).getOrThrow, 2)
    assertEquals(run(Runner.answer(day, "1", "R1000\nR50\n")).getOrThrow, "1")
    for (
      (input, expected) <- Seq(
                             "R50"       -> "1",
                             "L50"       -> "1",
                             "L50\nL1"   -> "1",
                             "R150"      -> "2",
                             "L150"      -> "2",
                             "R1000"     -> "10",
                             "L68\nR82"  -> "2",
                             "R50\nL100" -> "2",
                             "R50\nR100" -> "2"
                           )
    ) assertEquals(run(Runner.answer(day, "2", input)).getOrThrow, expected)
  }

  test("saved 2024 day 1 is discoverable and both official samples pass") {
    assertEquals(
      run(Runner.program(List("test", "2024", "1", "both"))).getOrThrow,
      ()
    )
  }

  test("saved 2024 day 2 remains discoverable but explicitly unfinished") {
    val day = Class
      .forName("aoc.solutions.y2024.Day02$")
      .getField("MODULE$")
      .get(null)
      .asInstanceOf[Puzzle]
    for (part <- Seq("1", "2")) {
      val error = intercept[IllegalArgumentException](
        run(Runner.answer(day, part, "")).getOrThrow
      )
      assert(error.getMessage.contains("unavailable"))
    }
  }
}
