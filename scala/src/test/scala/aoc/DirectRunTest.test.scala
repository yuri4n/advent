package aoc

import java.nio.file.Files
import kyo.*

class DirectRunTest extends PuzzleTest {
  private def run[A](value: => A < Puzzle.Effects): A = {
    import AllowUnsafe.embrace.danger
    KyoApp.Unsafe
      .runAndBlock(5.seconds)(Abort.catching[Throwable](value))
      .getOrThrow
  }

  test(
    "typed examples reject wrong answer types and support different answers explicitly"
  ) {
    assert(!scala.compiletime.testing.typeChecks("""
      import aoc.*
      import kyo.*
      val day = new Solution {
        type Prepared = String
        type Answer1 = Int
        def process(input: Input): String < Any = ""
        override val examples = Seq(Example1("input", "wrong type"))
      }
    """))
    val day = new Solution.WithAnswers {
      type Prepared = String
      type Answer1  = Int
      type Answer2  = String
      def process(input: Input): String < Puzzle.Effects =
        input.lines.fold("")(_ + _)
      override val part1                                 = solve((input: Prepared) => input.length)
      override val part2                                 = solve((input: Prepared) => input.reverse)
      override val examples                              = Seq(ExampleBoth("abc", 3, "cba"))
    }
    assertEquals(run(Runner.checkExamples(day, "both")), 2)
    assertEquals(
      run(Runner.auto(day, None)),
      Vector("Example 1 part 1" -> "3", "Example 1 part 2" -> "cba")
    )
  }

  test(
    "default solve-only replay materializes parsed records once without a snapshot override"
  ) {
    var parsed = 0
    val day    = new Solution {
      type Prepared = Stream[Int, Sync & Abort[ParseError]]
      type Answer1  = Int
      def process(input: Input): Prepared < Any =
        input.lines.map(line =>
          Parse.runOrAbort(line)(Parse.entireInput(Parse.int)).map { n =>
            parsed += 1; n
          }
        )
      override val part1                        = solve((values: Prepared) => values.fold(0)(_ + _))
    }
    val report = run(day.benchmark("1", "1\n2\n3\n", 3, 2))
    assertEquals(report.answer, "6")
    assertEquals(
      parsed,
      18
    ) // Five total passes plus one snapshot; no solve-only reparsing.
    assertEquals(report.timings.size, 3)
  }

  test("byte benchmarks do not decode UTF-8") {
    val day = new Solution {
      type Prepared = Stream[Byte, Sync]
      type Answer1  = Int
      def process(input: Input): Prepared < Any = input.bytes
      override val part1                        =
        solve((bytes: Prepared) => bytes.fold(0)((n, _) => n + 1))
    }
    assertEquals(
      run(day.benchmarkBytes("1", Array(255.toByte, 254.toByte), 2, 1)).answer,
      "2"
    )
  }

  test("nested character streams are snapshotted while each line is active") {
    val day = new Solution {
      type Prepared = Stream[Stream[Char, Sync], Sync]
      type Answer1  = Int
      def process(input: Input): Prepared < Any = input.lineChars
      override val part1                        = solve((rows: Prepared) =>
        rows.map(_.fold(0)((n, _) => n + 1)).fold(0)(_ + _)
      )
    }
    assertEquals(run(day.benchmark("1", "ab\n雪\n", 2, 1)).answer, "3")
  }

  test(
    "custom snapshot overrides automatic replay without affecting normal runs"
  ) {
    var snapshots = 0
    val day       = new Solution {
      type Prepared = Array[Int]
      type Answer1  = Int
      def process(input: Input): Prepared < Puzzle.Effects =
        input.lines.run.map(_.iterator.map(_.toInt).toArray)
      override val part1                                   = solve { (values: Prepared) =>
        values(0) += 1
        values.sum
      }
      override def snapshot(
          input: Prepared
      ): (() => Prepared) < Puzzle.Effects = {
        snapshots += 1
        () => input.clone()
      }
    }
    assertEquals(run(Runner.answer(day, "1", "1\n2")), "4")
    assertEquals(snapshots, 0)
    assertEquals(run(day.benchmark("1", "1\n2", 2, 1)).answer, "4")
    assertEquals(snapshots, 1)
  }

  test(
    "inner-type solutions run available parts and skip absent examples or files"
  ) {
    val day    = new Solution {
      type Prepared = Vector[Int]
      type Answer1  = Int
      def process(input: Input): Prepared < Puzzle.Effects =
        input.lines
          .map(line => Parse.runOrAbort(line)(Parse.entireInput(Parse.int)))
          .fold(Vector.empty[Int])(_ :+ _)
      override val part2                                   = solve((values: Prepared) => values.sum)
    }
    assert(day.isInstanceOf[KyoApp])
    assertEquals(day.availableParts, Seq("2"))
    assertEquals(run(Runner.auto(day, None)), Vector.empty[(String, String)])
    val path   = Files.createTempFile("aoc-input", ".txt")
    try {
      val _ = Files.writeString(path, "3\n4\n")
      assertEquals(
        run(Runner.auto(day, Some(path))),
        Vector("Input part 2" -> "7")
      )
    } finally { val _ = Files.deleteIfExists(path) }
    val report = run(day.benchmark("2", "3\n4\n", 2, 1))
    assertEquals(report.answer, "7")
    assertEquals(report.timings.size, 2)
  }
}
