package aoc

/** Inline sample expectations may cover either or both parts. */
final case class Sample(
    input: String,
    expected1: Option[String] = None,
    expected2: Option[String] = None
)

trait Day {
  def part1(input: String): Any
  def part2(input: String): Any
  def samples: Seq[Sample] = Seq.empty
}
