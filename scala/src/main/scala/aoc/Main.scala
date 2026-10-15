package aoc

import scala.util.control.NonFatal

object Main {
  private def load(year: String, day: String): Day = {
    require(
      year.matches("[0-9]{4}") && year.toInt >= 2015,
      "year must be at least 2015"
    )
    require(
      day.matches("[0-9]{1,2}") && day.toInt >= 1 && day.toInt <= 25,
      "day must be 1..25"
    )
    val name = f"aoc.solutions.y${year.toInt}.Day${day.toInt}%02d$$"
    try Class.forName(name).getField("MODULE$").get(null).asInstanceOf[Day]
    catch {
      case _: ClassNotFoundException =>
        throw IllegalArgumentException(
          s"No Scala solution for $year day $day; use aoc new first"
        )
    }
  }

  private def answer(day: Day, part: String, input: String): String =
    part match {
      case "1" => day.part1(input).toString
      case "2" => day.part2(input).toString
      case _   => throw IllegalArgumentException("part must be 1 or 2")
    }

  private def test(day: Day, part: String): Unit = {
    require(Set("1", "2", "both").contains(part), "part must be 1, 2, or both")
    var checked = 0
    day.samples.zipWithIndex.foreach { case (sample, index) =>
      Seq("1" -> sample.expected1, "2" -> sample.expected2).foreach {
        case (number, Some(expected)) if part == "both" || part == number =>
          val actual = answer(day, number, sample.input)
          require(
            actual == expected,
            s"Sample ${index + 1} part $number: expected $expected, got $actual"
          )
          checked += 1
        case _                                                            => ()
      }
    }
    require(
      checked > 0,
      "No inline sample expectations for the selected part; add a Sample before testing"
    )
    println(s"Passed $checked Scala sample assertion(s)")
  }

  def main(args: Array[String]): Unit = {
    try {
      args.toList match {
        case List("run", year, day, part)  =>
          println(answer(load(year, day), part, scala.io.Source.stdin.mkString))
        case List("test", year, day)       => test(load(year, day), "both")
        case List("test", year, day, part) => test(load(year, day), part)
        case _                             =>
          throw IllegalArgumentException(
            "Usage: run YEAR DAY PART | test YEAR DAY [1|2|both]"
          )
      }
    } catch {
      case error: NotImplementedError =>
        System.err.println(
          s"Scala solution is an unfinished stub: ${error.getMessage}"
        )
        sys.exit(1)
      case NonFatal(error)            =>
        System.err.println(s"Scala: ${error.getMessage}")
        sys.exit(1)
    }
  }
}
