package aoc.solutions.y2021

import aoc.{ExampleBoth, Input, Puzzle, Solution}
import kyo.*
import scala.annotation.tailrec

object Day01 extends Solution {
  type Prepared = Vector[Int]
  type Answer1  = Int
  def process(input: Input): Vector[Int] < Puzzle.Effects =
    input.lines
      .map(line => Parse.runOrAbort(line)(Parse.entireInput(Parse.int)))
      .fold(Vector.empty[Int])(_ :+ _)

  override val part1 = solve((input: Prepared) => countIncreases(input))
  override val part2 =
    solve((input: Prepared) => countIncreases(mapTriples(input)))

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

  override val examples = Seq(
    ExampleBoth(
      "199\n200\n208\n210\n200\n207\n240\n269\n260\n263\n",
      7,
      5
    )
  )
}
