package aoc

import kyo.*

/** Expectations carry the corresponding part's native answer type. */
sealed trait Example[+A, +B]    {
  def input: String
  def expected1: Option[A]
  def expected2: Option[B]
}
final case class Example1[A](input: String, expected: A)
    extends Example[A, Nothing] {
  def expected1 = Some(expected)
  def expected2 = None
}
final case class Example2[B](input: String, expected: B)
    extends Example[Nothing, B] {
  def expected1 = None
  def expected2 = Some(expected)
}
final case class ExampleBoth[A, B](input: String, first: A, second: B)
    extends Example[A, B]       {
  def expected1 = Some(first)
  def expected2 = Some(second)
}
final case class ExampleInput(input: String) extends Example[Nothing, Nothing] {
  def expected1 = None
  def expected2 = None
}

// Source compatibility for the original string-based Sample constructor.
object Example {
  def apply[A, B](
      input: String,
      expected1: Option[A] = None,
      expected2: Option[B] = None
  ): Example[A, B] =
    (expected1, expected2) match {
      case (Some(a), Some(b)) => ExampleBoth(input, a, b)
      case (Some(a), None)    => Example1(input, a)
      case (None, Some(b))    => Example2(input, b)
      case _                  => ExampleInput(input)
    }
}
type Sample = Example[String, String]
val Sample: Example.type = Example

object Puzzle {
  type Effects = Async & Abort[Throwable] & Scope
}

/** The runner's common contract. Use Day for ordinary text input. */
sealed trait Puzzle {
  def samples: Seq[Sample]         = Seq.empty
  def examples: Seq[Example[?, ?]] = samples
  def availableParts: Seq[String]  = Seq("1", "2")
  private[aoc] def answer(part: String, input: Input): String < Puzzle.Effects
  private[aoc] def runExample(
      part: String,
      index: Int
  ): String < Puzzle.Effects = {
    val example  = examples(index)
    val expected = (if (part == "1") example.expected1 else example.expected2)
      .map(_.toString)
    answer(part, Input.fromText(example.input)).map { actual =>
      require(
        expected.forall(_ == actual),
        s"Example ${index + 1} part $part: expected ${expected.getOrElse("")}, got $actual"
      )
      actual
    }
  }
}

/** Stream-first processing shared by both parts. Prepared may be a lazy stream
  * or an explicitly buffered value. Each execution receives fresh input.
  */
object Solution {
  abstract class WithAnswers extends PuzzleApp with Puzzle {
    export Parsing.nonNegativeLong

    type Prepared
    type Answer1
    type Answer2
    type Solver[A] = Prepared => A < Puzzle.Effects
    override def examples: Seq[Example[Answer1, Answer2]] = Seq.empty

    def process(input: Input): Prepared < Puzzle.Effects
    def part1: Option[Solver[Answer1]] = None
    def part2: Option[Solver[Answer2]] = None

    /** Register only implemented parts; omitted parts are unavailable, not
      * failures.
      */
    protected final def solve[A](body: Solver[A])(using
        replay: Replay[Prepared]
    ): Option[Solver[A]] = {
      replayInput = replay.apply
      Some(body)
    }

    final override def availableParts: Seq[String] =
      Seq("1" -> part1.isDefined, "2" -> part2.isDefined).collect {
        case (part, true) => part
      }

    private var replayInput: Prepared => (() => Prepared) < Puzzle.Effects =
      Replay.value[Prepared].apply

    /** Benchmark-only customization. Registered parts automatically reuse
      * strict immutable input or materialize Kyo streams once for replay.
      * Override for custom data shapes or mutable state. Normal runs never
      * invoke this hook.
      */
    def snapshot(input: Prepared): (() => Prepared) < Puzzle.Effects =
      replayInput(input)

    private[aoc] final def preparedAnswer(
        part: String,
        input: Prepared
    ): String < Puzzle.Effects = Sync.defer {
      part match {
        case "1" =>
          part1
            .getOrElse(throw IllegalArgumentException("part 1 is unavailable"))(
              input
            )
            .map(_.toString)
        case "2" =>
          part2
            .getOrElse(throw IllegalArgumentException("part 2 is unavailable"))(
              input
            )
            .map(_.toString)
        case _   => throw IllegalArgumentException("part must be 1 or 2")
      }
    }

    private[aoc] final def answer(
        part: String,
        input: Input
    ): String < Puzzle.Effects = Sync.defer {
      require(availableParts.contains(part), s"part $part is unavailable")
      process(input).flatMap(preparedAnswer(part, _))
    }

    private[aoc] final override def runExample(
        part: String,
        index: Int
    ): String < Puzzle.Effects = Sync.defer {
      val example                                          = examples(index)
      def check[A](expected: Option[A], actual: A): String = {
        given CanEqual[A, A] = CanEqual.derived
        require(
          expected.forall(_ == actual),
          s"Example ${index + 1} part $part: expected ${expected.fold("")(_.toString)}, got $actual"
        )
        actual.toString
      }
      require(availableParts.contains(part), s"part $part is unavailable")
      process(Input.fromText(example.input)).flatMap { prepared =>
        if (part == "1")
          part1.get(prepared).map(value => check(example.expected1, value))
        else part2.get(prepared).map(value => check(example.expected2, value))
      }
    }

    private[aoc] final def benchmark(
        part: String,
        text: String,
        iterations: Int,
        warmup: Int
    ): Benchmark.Report < Puzzle.Effects =
      benchmarkBytes(
        part,
        text.getBytes(java.nio.charset.StandardCharsets.UTF_8),
        iterations,
        warmup
      )

    private[aoc] final def benchmarkBytes(
        part: String,
        bytes: Array[Byte],
        iterations: Int,
        warmup: Int
    ): Benchmark.Report < Puzzle.Effects =
      Benchmark.plan(this, part, bytes, iterations, warmup)

    run(Abort.catching[Throwable] {
      Sync.defer {
        require(
          args.isEmpty,
          "Run a day without arguments in the IDE; use aoc for explicit input/part selection"
        )
        Runner.auto(this, Runner.defaultInput(this)).flatMap { results =>
          Kyo
            .foreach(results) { case (label, answer) =>
              Console.printLine(s"$label: $answer")
            }
            .unit
        }
      }
    })
  }
}

/** Most AoC parts return the same answer type. Use Solution.WithAnswers to
  * declare different Answer1 and Answer2 types without duplicating the runner.
  */
abstract class Solution extends Solution.WithAnswers {
  type Answer2 = Answer1
}

/** Compatibility API for text-based puzzles. New solutions use Solution. */
trait Day[Part1, Part2] extends Puzzle {
  def part1(input: String): Part1 < Puzzle.Effects
  def part2(input: String): Part2 < Puzzle.Effects

  private[aoc] final def answer(
      part: String,
      input: Input
  ): String < Puzzle.Effects =
    input.text.flatMap { text =>
      part match {
        case "1" => part1(text).map(_.toString)
        case "2" => part2(text).map(_.toString)
        case _   => throw IllegalArgumentException("part must be 1 or 2")
      }
    }
}

/** Single-pass input streams with the same optional effects as Day. */
trait StreamingDay[Part1, Part2] extends Puzzle {
  def part1(input: Input): Part1 < Puzzle.Effects
  def part2(input: Input): Part2 < Puzzle.Effects

  private[aoc] final def answer(
      part: String,
      input: Input
  ): String < Puzzle.Effects =
    Sync.defer {
      part match {
        case "1" => part1(input).map(_.toString)
        case "2" => part2(input).map(_.toString)
        case _   => throw IllegalArgumentException("part must be 1 or 2")
      }
    }
}
