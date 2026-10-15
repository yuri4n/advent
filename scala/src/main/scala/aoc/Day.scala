package aoc

import kyo.{<, Abort, Async, Resource}

/** Inline sample expectations may cover either or both parts. */
final case class Sample(
    input: String,
    expected1: Option[String] = None,
    expected2: Option[String] = None
)

object Day {

  /** The shared runner handles these effects; individual parts can use fewer.
    */
  type Effects = Async & Abort[Throwable] & Resource
}

trait Day[Part1, Part2] {
  def part1(input: String): Part1 < Day.Effects
  def part2(input: String): Part2 < Day.Effects
  def samples: Seq[Sample] = Seq.empty
}
