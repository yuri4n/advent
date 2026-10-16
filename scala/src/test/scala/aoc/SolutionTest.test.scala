package aoc

import kyo.*

class SolutionTest extends PuzzleTest {
  private def run[A](value: => A < Puzzle.Effects): A = {
    import AllowUnsafe.embrace.danger
    KyoApp.Unsafe
      .runAndBlock(5.seconds)(Abort.catching[Throwable](value))
      .getOrThrow
  }

  test(
    "examples use shared buffered preprocessing and can run without expectations"
  ) {
    val day = new Solution {
      type Prepared = Vector[Int]
      type Answer1  = Int
      def process(input: Input): Vector[Int] < Puzzle.Effects =
        input.lines
          .map(line => Parse.runOrAbort(line)(Parse.entireInput(Parse.int)))
          .fold(Vector.empty[Int])(_ :+ _)
      override val part1                                      = solve((values: Prepared) => values.sum)
      override val part2                                      = solve((values: Prepared) => values.product)
      override val examples                                   =
        Seq(Example1("2\n3\n", 5), ExampleInput("  untouched\n\n"))
    }
    assertEquals(run(Runner.checkExamples(day, "1")), 1)
    assertEquals(Runner.exampleInput(day, 2), "  untouched\n\n")
    assert(
      intercept[IllegalArgumentException](
        Runner.exampleInput(day, 0)
      ).getMessage.contains("does not exist")
    )
    assert(
      intercept[IllegalArgumentException](
        Runner.exampleInput(day, 3)
      ).getMessage.contains("does not exist")
    )
    intercept[ParseError](
      run(Runner.answer(day, "1", "9223372036854775808\n"))
    )
  }

  test("shared preprocessing remains lazy and gets fresh lines for each part") {
    var processed = 0
    val day       = new Solution {
      type Prepared = Stream[Int, Sync & Abort[ParseError]]
      type Answer1  = Int
      def process(
          input: Input
      ): Stream[Int, Sync & Abort[ParseError]] < Any = {
        processed += 1
        input.lines.map(line =>
          Parse.runOrAbort(line)(Parse.entireInput(Parse.int))
        )
      }
      override val part1 =
        solve((values: Prepared) => values.take(1).fold(0)(_ + _))
      override val part2 = solve((values: Prepared) => values.fold(0)(_ + _))
    }
    val first     = Runner.answer(day, "1", "7\ninvalid\n")
    assertEquals(processed, 0)
    assertEquals(run(first), "7")
    assertEquals(run(first), "7")
    assertEquals(run(Runner.answer(day, "2", "7\n3\n")), "10")
    assertEquals(processed, 3)
    intercept[ParseError](run(Runner.answer(day, "2", "7junk\n")))
  }
}
