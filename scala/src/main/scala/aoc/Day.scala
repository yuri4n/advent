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

/** The runner's common contract. Use Day for ordinary text input. */
sealed trait Puzzle {
  def samples: Seq[Sample] = Seq.empty
  private[aoc] def answer(part: String, input: Input): String < Puzzle.Effects
}

/** Text-based puzzles with pure values or optional Kyo effects. */
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
    IO {
      part match {
        case "1" => part1(input).map(_.toString)
        case "2" => part2(input).map(_.toString)
        case _   => throw IllegalArgumentException("part must be 1 or 2")
      }
    }
}
