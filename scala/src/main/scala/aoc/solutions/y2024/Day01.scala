package aoc.solutions.y2024

import aoc.{Day, Sample}
import kyo.<

object Day01 extends Day[Int, Int] {
  def sortZip[A: Ordering, B: Ordering](
      seqA: Seq[A],
      seqB: Seq[B]
  ): Seq[(A, B)] =
    seqA.sorted.zip(seqB.sorted)

  def first(seq: Vector[(Int, Int)]): Int = {
    val (left, right) = seq.unzip
    sortZip(left, right).map { case (a, b) => math.abs(a - b) }.sum
  }

  def second(seq: Vector[(Int, Int)]): Int = {
    val (left, right) = seq.unzip
    val rightCount    = right.foldLeft(Map.empty[Int, Int]) { (counts, value) =>
      counts + (value -> (counts.getOrElse(value, 0) + 1))
    }
    left.map(value => value * rightCount.getOrElse(value, 0)).sum
  }

  private def parse(input: String): Vector[(Int, Int)] =
    input.linesIterator.map { line =>
      line.split("\\s+") match {
        case Array(a, b) => (a.toInt, b.toInt)
        case _           =>
          throw IllegalArgumentException("expected two location IDs per line")
      }
    }.toVector

  def part1(input: String): Int < Any = first(parse(input))
  def part2(input: String): Int < Any = second(parse(input))

  override val samples = Seq(
    Sample("3   4\n4   3\n2   5\n1   3\n3   9\n3   3\n", Some("11"), Some("31"))
  )
}
