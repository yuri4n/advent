package aoc

import java.util.concurrent.atomic.AtomicInteger
import kyo.*

class RunnerTest extends munit.FunSuite {
  private def run[A](value: => A < Puzzle.Effects): Result[Throwable, A] = {
    import AllowUnsafe.embrace.danger
    KyoApp.Unsafe.runAndBlock(5.seconds)(Abort.catching[Throwable](value))
  }

  test(
    "pure bodies need no IO wrapper and the runner defers their invocation"
  ) {
    def nextNumber(input: String): Int = input.toInt + 1
    assertEquals(nextNumber("41"), 42)
    var requested                      = 0
    val day                            = new Day[Int, String] {
      def part1(input: String): Int < Any    = {
        requested += 1
        nextNumber(input)
      }
      def part2(input: String): String < Any = s"answer: $input"
    }
    val answer                         = Runner.answer(day, "1", "41")
    assertEquals(requested, 0)
    assertEquals(run(answer).getOrThrow, "42")
    assertEquals(requested, 1)
    assertEquals(run(answer).getOrThrow, "42")
    assertEquals(requested, 2)
    assertEquals(
      run(Runner.answer(day, "2", "text")).getOrThrow,
      "answer: text"
    )
  }

  test(
    "streaming solutions defer both invocation and effects until execution"
  ) {
    var requested = 0
    var executed  = 0
    val day       = new StreamingDay[Int, String] {
      def part1(input: Input): Int < IO    = {
        requested += 1
        input.text.map { text =>
          executed += 1
          text.toInt + 1
        }
      }
      def part2(input: Input): String < IO =
        input.text.map(text => s"answer: $text")
    }
    val answer    = Runner.answer(day, "1", "41")
    assertEquals((requested, executed), (0, 0))
    assertEquals(run(answer).getOrThrow, "42")
    assertEquals((requested, executed), (1, 1))
    assertEquals(
      run(Runner.answer(day, "2", "text")).getOrThrow,
      "answer: text"
    )
  }

  test(
    "ordinary days support bounded parallel effects without streaming input"
  ) {
    val active  = new AtomicInteger(0)
    val maximum = new AtomicInteger(0)
    val day     = new Day[Long, Long] {
      def part1(input: String): Long < Async =
        Barrier.init(2).flatMap { barrier =>
          Async
            .foreach(
              input.linesIterator.map(_.toLong).toVector,
              concurrency = 2
            ) { number =>
              for {
                _     <- IO {
                           val count = active.incrementAndGet()
                           assert(count <= 2)
                           maximum.accumulateAndGet(
                             count,
                             (before, current) => math.max(before, current)
                           )
                         }
                // The first worker cannot finish until a second worker overlaps.
                _     <- barrier.await
                value <- IO {
                           val remaining = active.decrementAndGet()
                           assert(remaining >= 0)
                           number
                         }
              } yield value
            }
            .map(_.sum)
        }
      def part2(input: String): Long < Any   = 0L
    }
    assertEquals(
      run(Runner.answer(day, "1", "2147483648\n1\n2\n3")).getOrThrow,
      "2147483654"
    )
    assertEquals(maximum.get(), 2)
    assertEquals(active.get(), 0)
  }

  test("ordinary-day typed Abort failures reach the runtime boundary") {
    val failure = IllegalArgumentException("invalid puzzle input")
    val day     = new Day[Int, Int] {
      def part1(input: String): Int < Abort[Throwable] = Abort.fail(failure)
      def part2(input: String): Int < Any              = 0
    }
    val result  = run(Runner.answer(day, "1", "invalid"))
    assert(result.isFailure)
    val caught  = intercept[IllegalArgumentException](result.getOrThrow)
    assert(caught eq failure)
  }

  test("pure parse errors and unfinished stubs remain failures") {
    val day        = new Day[Int, Int] {
      def part1(input: String): Int < Any = input.toInt
      def part2(input: String): Int < Any = ???
    }
    val parseError = intercept[NumberFormatException](
      run(Runner.answer(day, "1", "invalid")).getOrThrow
    )
    assert(parseError.getMessage.contains("invalid"))
    val stubError  = intercept[NotImplementedError](
      run(Runner.answer(day, "2", "")).getOrThrow
    )
    assert(stubError.getMessage.contains("implementation is missing"))
  }

  test("pure samples sequence both parts and honor selection") {
    var calls = Vector.empty[String]
    val day   = new Day[Int, Long] {
      def part1(input: String): Int < Any  = {
        calls = calls :+ s"1:$input"
        input.toInt
      }
      def part2(input: String): Long < Any = {
        calls = calls :+ s"2:$input"
        input.toLong * 10
      }
      override val samples                 = Seq(
        Sample("2", Some("2"), Some("20")),
        Sample("3", Some("3"))
      )
    }
    assertEquals(run(Runner.checkSamples(day, "both")).getOrThrow, 3)
    assertEquals(calls, Vector("1:2", "2:2", "1:3"))
    calls = Vector.empty
    assertEquals(run(Runner.checkSamples(day, "2")).getOrThrow, 1)
    assertEquals(calls, Vector("2:2"))
  }

  test("sample mismatches identify the sample and stop further execution") {
    var executed = 0
    val day      = new Day[Int, Int] {
      def part1(input: String): Int < Any = {
        executed += 1
        input.toInt
      }
      def part2(input: String): Int < Any = 0
      override val samples                = Seq(Sample("2", Some("3")), Sample("4", Some("4")))
    }
    val error    = intercept[IllegalArgumentException](
      run(Runner.checkSamples(day, "1")).getOrThrow
    )
    assert(error.getMessage.contains("Sample 1 part 1: expected 3, got 2"))
    assertEquals(executed, 1)
  }

  test("testing both requires expectations for both parts") {
    val day = new Day[Int, Int] {
      def part1(input: String): Int < Any = 1
      def part2(input: String): Int < Any = 2
      override val samples                = Seq(Sample("", Some("1")))
    }
    assertEquals(run(Runner.checkSamples(day, "1")).getOrThrow, 1)
    for (selection <- Seq("2", "both")) {
      val error = intercept[IllegalArgumentException](
        run(Runner.checkSamples(day, selection)).getOrThrow
      )
      assert(
        error.getMessage.contains("No inline sample expectations for part(s) 2")
      )
    }
  }

  test("ordinary-day resource finalizers run after success and failure") {
    var released = 0
    val day      = new Day[Int, Int] {
      def part1(input: String): Int < (Resource & IO) =
        Resource.ensure(IO { released += 1 }).andThen(IO(input.toInt))
      def part2(input: String): Int < Any             = 0
    }
    assertEquals(run(Runner.answer(day, "1", "42")).getOrThrow, "42")
    assertEquals(released, 1)
    val error    = intercept[NumberFormatException](
      run(Runner.answer(day, "1", "invalid")).getOrThrow
    )
    assert(error.getMessage.contains("invalid"))
    assertEquals(released, 2)
  }

  test(
    "streaming samples get fresh input for every part, sample, and execution"
  ) {
    val day    = new StreamingDay[Int, Int] {
      def part1(input: Input): Int < IO =
        input.lines.fold(0)((count, _) => count + 1)
      def part2(input: Input): Int < IO =
        input.bytes.fold(0)((count, _) => count + 1)
      override val samples              = Seq(
        Sample("é\n猫\n", Some("2"), Some("7")),
        Sample("🙂", Some("1"), Some("4")),
        Sample("", Some("0"), Some("0"))
      )
    }
    assertEquals(run(Runner.checkSamples(day, "both")).getOrThrow, 6)
    val answer = Runner.answer(day, "2", "é\n猫\n")
    assertEquals(run(answer).getOrThrow, "7")
    assertEquals(run(answer).getOrThrow, "7")
  }

  test(
    "unselected failing parts are never invoked for pure or streaming solutions"
  ) {
    val pure      = new Day[Int, Int] {
      def part1(input: String): Int < Any = input.toInt
      def part2(input: String): Int < Any =
        throw IllegalArgumentException("unselected pure part")
      override val samples                = Seq(Sample("7", Some("7"), Some("unused")))
    }
    val streaming = new StreamingDay[Int, Int] {
      def part1(input: Input): Int < IO               = input.text.map(_.toInt)
      def part2(input: Input): Int < Abort[Throwable] =
        Abort.fail(IllegalArgumentException("unselected streaming part"))
      override val samples                            = Seq(Sample("7", Some("7"), Some("unused")))
    }
    for (day <- Seq[Puzzle](pure, streaming)) {
      assertEquals(run(Runner.checkSamples(day, "1")).getOrThrow, 1)
      assertEquals(run(Runner.answer(day, "1", "7")).getOrThrow, "7")
    }
  }
}
