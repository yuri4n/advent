package aoc.solutions.y2021

import aoc.{Day, Sample}
import kyo.{<, IO}

object Day02 extends Day[Int, Int] {
  type InstructionLine = (Instruction, Int)

  def part1(input: String): Int < IO = IO {
    val pos = findPos(parse(input))
    pos._1 * pos._2
  }

  def part2(input: String): Int < IO = IO {
    val pos = findPosWithAim(parse(input))
    pos._1 * pos._2
  }

  def parse(input: String): Seq[InstructionLine] =
    input.linesIterator.map { s =>
      val ss = s.split(' ')
      (Instruction.valueOf(ss(0).capitalize), ss(1).toInt)
    }.toVector

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

  override val samples = Seq(
    Sample(
      "forward 5\ndown 5\nforward 8\nup 3\ndown 8\nforward 2\n",
      Some("150"),
      Some("900")
    )
  )
}
