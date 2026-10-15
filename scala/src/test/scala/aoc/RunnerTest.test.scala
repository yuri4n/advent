package aoc

import kyo.*

class RunnerTest extends munit.FunSuite {
  private def run[A](value: => A < Day.Effects): Result[Throwable, A] = {
    import AllowUnsafe.embrace.danger
    KyoApp.Unsafe.runAndBlock(5.seconds)(Abort.catching[Throwable](value))
  }

  test(
    "answers defer and execute solution effects, preserving numeric and text results"
  ) {
    var requested = 0
    var executed  = 0
    val day       = new Day[Int, String] {
      def part1(input: String): Int < IO    = {
        requested += 1
        IO {
          executed += 1
          input.toInt + 1
        }
      }
      def part2(input: String): String < IO = IO(s"answer: $input")
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

  test("answers await asynchronous computations") {
    val day = new Day[Long, Long] {
      def part1(input: String): Long < Async =
        Async.delay(1.millis)(IO(input.toLong + 1))
      def part2(input: String): Long < IO    = IO(0L)
    }
    assertEquals(
      run(Runner.answer(day, "1", "2147483648")).getOrThrow,
      "2147483649"
    )
  }

  test("typed Abort failures reach the runtime boundary") {
    val failure = IllegalArgumentException("invalid puzzle input")
    val day     = new Day[Int, Int] {
      def part1(input: String): Int < Abort[Throwable] = Abort.fail(failure)
      def part2(input: String): Int < IO               = IO(0)
    }
    val result  = run(Runner.answer(day, "1", ""))
    assert(result.isFailure)
    val caught  = intercept[IllegalArgumentException](result.getOrThrow)
    assert(caught eq failure)
  }

  test("thrown parse errors and unfinished stubs remain failures") {
    val day        = new Day[Int, Int] {
      def part1(input: String): Int < IO = IO(input.toInt)
      def part2(input: String): Int < IO = IO(???)
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

  test("samples sequence both parts and honor selection") {
    var calls = Vector.empty[String]
    val day   = new Day[Int, Long] {
      def part1(input: String): Int < IO  = IO {
        calls = calls :+ s"1:$input"
        input.toInt
      }
      def part2(input: String): Long < IO = IO {
        calls = calls :+ s"2:$input"
        input.toLong * 10
      }
      override val samples                = Seq(
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
      def part1(input: String): Int < IO = IO {
        executed += 1
        input.toInt
      }
      def part2(input: String): Int < IO = IO(0)
      override val samples               = Seq(Sample("2", Some("3")), Sample("4", Some("4")))
    }
    val error    = intercept[IllegalArgumentException](
      run(Runner.checkSamples(day, "1")).getOrThrow
    )
    assert(error.getMessage.contains("Sample 1 part 1: expected 3, got 2"))
    assertEquals(executed, 1)
  }

  test("testing both requires expectations for both parts") {
    val day = new Day[Int, Int] {
      def part1(input: String): Int < IO = IO(1)
      def part2(input: String): Int < IO = IO(2)
      override val samples               = Seq(Sample("", Some("1")))
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

  test("resource finalizers run after successful and failed solutions") {
    var released = 0
    val day      = new Day[Int, Int] {
      def part1(input: String): Int < (Resource & IO) =
        Resource.ensure(IO { released += 1 }).andThen(IO(input.toInt))
      def part2(input: String): Int < IO              = IO(0)
    }
    assertEquals(run(Runner.answer(day, "1", "42")).getOrThrow, "42")
    assertEquals(released, 1)
    val error    = intercept[NumberFormatException](
      run(Runner.answer(day, "1", "invalid")).getOrThrow
    )
    assert(error.getMessage.contains("invalid"))
    assertEquals(released, 2)
  }
}
