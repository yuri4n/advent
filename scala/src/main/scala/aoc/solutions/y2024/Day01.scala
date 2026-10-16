package aoc.solutions.y2024

import aoc.{ExampleBoth, Input, Puzzle, Solution}
import kyo.*

object Day01 extends Solution {
  type Prepared = Vector[(Int, Int)]
  type Answer1  = Int
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

  def process(input: Input): Vector[(Int, Int)] < Puzzle.Effects =
    input.lines
      .map(line =>
        Parse.runOrAbort(line)(
          Parse.entireInput(Parse.inOrder(Parse.int, Parse.spaced(Parse.int)))
        )
      )
      .fold(Vector.empty[(Int, Int)])(_ :+ _)

  override val part1 = solve((input: Prepared) => first(input))
  override val part2 = solve((input: Prepared) => second(input))

  override val examples = Seq(
    ExampleBoth("3   4\n4   3\n2   5\n1   3\n3   9\n3   3\n", 11, 31)
  )
}
