package aoc.solutions.y2021

import aoc.{Day, Sample}
import scala.annotation.tailrec

object Day01 extends Day {
  def part1(input: String): Int = countIncreases(
    input.linesIterator.map(_.toInt).toVector
  )

  def part2(input: String): Int =
    countIncreases(mapTriples(input.linesIterator.map(_.toInt).toVector))

  def countIncreases(depthsList: Seq[Int]): Int =
    depthsList.zip(depthsList.tail).foldRight(0) { (pair, acc) =>
      if (pair._1 < pair._2) acc + 1 else acc
    }

  def mapTriples(depthLists: Seq[Int]): Seq[Int] = {
    @tailrec
    def exec(res: Seq[Int], pending: Seq[Int]): Seq[Int] = pending match {
      case Seq(x, y, z)     => res :+ (x + y + z)
      case Seq(x, y, z, _*) => exec(res :+ (x + y + z), pending.tail)
      case xs               => throw Exception(s"xs has less than 3 elements $xs")
    }
    exec(Vector(), depthLists)
  }

  override val samples = Seq(
    Sample(
      "199\n200\n208\n210\n200\n207\n240\n269\n260\n263\n",
      Some("7"),
      Some("5")
    )
  )
}
