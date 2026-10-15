package aoc

import kyo.{<, Abort, Async, IO, Resource}

/** Inline sample expectations may cover either or both parts. */
final case class Sample(
    input: String,
    expected1: Option[String] = None,
    expected2: Option[String] = None
)

object Puzzle {
  type Effects = Async & Abort[Throwable] & Resource
}

/** The runner's common contract. Use Day for ordinary pure solutions. */
sealed trait Puzzle {
  def samples: Seq[Sample] = Seq.empty
  private[aoc] def answer(part: String, input: Input): String < Puzzle.Effects
}

/** Pure puzzle functions; the Kyo runner owns input and execution. */
trait Day[Part1, Part2] extends Puzzle {
  def part1(input: String): Part1
  def part2(input: String): Part2

  private[aoc] final def answer(
      part: String,
      input: Input
  ): String < Puzzle.Effects =
    input.text.map { text =>
      part match {
        case "1" => part1(text).toString
        case "2" => part2(text).toString
        case _   => throw IllegalArgumentException("part must be 1 or 2")
      }
    }
}

/** Opt in to single-pass line/byte streams or other Kyo effects when needed. */
trait StreamingDay[Part1, Part2] extends Puzzle {
  def part1(input: Input): Part1 < Puzzle.Effects
  def part2(input: Input): Part2 < Puzzle.Effects

  private[aoc] final def answer(
      part: String,
      input: Input
  ): String < Puzzle.Effects =
    IO {
      part match {
        case "1" => part1(input).map(_.toString)
        case "2" => part2(input).map(_.toString)
        case _   => throw IllegalArgumentException("part must be 1 or 2")
      }
    }
}
