package aoc.solutions.y2021

import aoc.{ExampleBoth, Input, Puzzle, Solution}
import kyo.*

object Day02 extends Solution {
  type Prepared        = Vector[InstructionLine]
  type Answer1         = Int
  type InstructionLine = (Instruction, Int)

  override val part1 = solve { (input: Prepared) =>
    val pos = findPos(input)
    pos._1 * pos._2
  }

  override val part2 = solve { (input: Prepared) =>
    val pos = findPosWithAim(input)
    pos._1 * pos._2
  }

  def process(
      input: Input
  ): Vector[InstructionLine] < Puzzle.Effects =
    input.lines
      .map { line =>
        Parse.runOrAbort(line) {
          Parse.entireInput {
            Parse
              .inOrder(Parse.identifier, Parse.spaced(Parse.int))
              .map { case (instruction, value) =>
                (Instruction.valueOf(instruction.capitalize), value)
              }
          }
        }
      }
      .fold(Vector.empty[InstructionLine])(_ :+ _)

  def findPos(list: Seq[InstructionLine]): (Int, Int) =
    list.foldRight((0, 0)) {
      case ((Instruction.Forward, v), acc) => (acc._1 + v, acc._2)
      case ((Instruction.Down, v), acc)    => (acc._1, acc._2 + v)
      case ((Instruction.Up, v), acc)      => (acc._1, acc._2 - v)
    }

  def findPosWithAim(list: Seq[InstructionLine]): (Int, Int, Int) =
    list.foldLeft((0, 0, 0)) {
      case ((horizontal, depth, aim), (Instruction.Forward, v)) =>
        (horizontal + v, depth + (aim * v), aim)
      case ((horizontal, depth, aim), (Instruction.Down, v))    =>
        (horizontal, depth, aim + v)
      case ((horizontal, depth, aim), (Instruction.Up, v))      =>
        (horizontal, depth, aim - v)
    }

  enum Instruction derives CanEqual {
    case Forward, Down, Up
  }

  override val examples = Seq(
    ExampleBoth(
      "forward 5\ndown 5\nforward 8\nup 3\ndown 8\nforward 2\n",
      150,
      900
    )
  )
}
