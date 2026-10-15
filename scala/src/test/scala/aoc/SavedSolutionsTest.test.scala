package aoc

import kyo.*

class SavedSolutionsTest extends munit.FunSuite {
  private def run[A](value: => A < Puzzle.Effects): Result[Throwable, A] = {
    import AllowUnsafe.embrace.danger
    KyoApp.Unsafe.runAndBlock(5.seconds)(Abort.catching[Throwable](value))
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
      val error = intercept[NotImplementedError](
        run(Runner.answer(day, part, "")).getOrThrow
      )
      assert(error.getMessage.contains("implementation is missing"))
    }
  }
}
