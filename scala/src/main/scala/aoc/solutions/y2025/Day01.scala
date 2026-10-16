package aoc.solutions.y2025

import aoc.{Example, ExampleBoth, Input, Puzzle, Solution}
import kyo.*

object Day01 extends Solution {
  type Prepared = Vector[(Direction, Int)]
  type Answer1  = Long

  final val startingPoint = 50
  final val maxPoint      = 100

  enum Direction derives CanEqual:
    case L, R

  def process(
      input: Input
  ): Vector[(Direction, Int)] < Puzzle.Effects =
    input.lines
      .map { line =>
        Parse.runOrAbort(line) {
          Parse.entireInput {
            Parse
              .inOrder(Parse.anyIn("LR"), Parse.int)
              .map { case (direction, steps) =>
                (Direction.valueOf(direction.toString), steps)
              }
          }
        }
      }
      .fold(Vector.empty[(Direction, Int)])(_ :+ _)

  override val part1 = solve { (input: Prepared) =>
    input
      .scanLeft(startingPoint) { case (acc, (direction, steps)) =>
        direction match
          case Direction.L => Math.floorMod(acc - steps, maxPoint)
          case Direction.R => Math.floorMod(acc + steps, maxPoint)
      }
      .count(_ == 0)
      .toLong
  }

  override val part2 = solve { (input: Prepared) =>
    input
      .foldLeft((0, startingPoint)) {
        case ((acc, currentPoint), (direction, steps)) =>
          val fullTurns = steps / maxPoint
          val remaining = steps % maxPoint

          val newAcc =
            acc + fullTurns + (if (
                                   direction match
                                     case Direction.R =>
                                       currentPoint + remaining >= maxPoint
                                     case Direction.L =>
                                       currentPoint != 0 && remaining >= currentPoint
                                 )
                               then 1
                               else 0)

          val newPoint = direction match
            case Direction.L => Math.floorMod(currentPoint - steps, maxPoint)
            case Direction.R => Math.floorMod(currentPoint + steps, maxPoint)

          (newAcc, newPoint)
      }
      ._1
      .toLong
  }

  override val examples: Seq[Example[Answer1, Answer2]] = Seq(
    ExampleBoth(
      """L68
        |L30
        |R48
        |L5
        |R60
        |L55
        |L1
        |L99
        |R14
        |L82
        |""".stripMargin,
      3L,
      6L
    )
  )
}
