package aoc

import java.io.ByteArrayInputStream
import java.nio.charset.StandardCharsets.UTF_8
import kyo.*

private[aoc] object Runner {
  private def load(year: String, day: String): Puzzle = {
    require(
      year.matches("[0-9]{4}") && year.toInt >= 2015 && year.toInt <= 2099,
      "year must be 2015..2099"
    )
    require(
      day.matches("[0-9]{1,2}") && day.toInt >= 1 && day.toInt <= 25,
      "day must be 1..25"
    )
    val name = f"aoc.solutions.y${year.toInt}.Day${day.toInt}%02d$$"
    try
      Class.forName(name).getField("MODULE$").get(null).asInstanceOf[Puzzle]
    catch {
      case _: ClassNotFoundException =>
        throw IllegalArgumentException(
          s"No Scala solution for $year day $day; use aoc new first"
        )
    }
  }

  private def answerInput(
      day: Puzzle,
      part: String,
      input: Input
  ): String < Puzzle.Effects =
    IO {
      require(part == "1" || part == "2", "part must be 1 or 2")
      day.answer(part, input)
    }

  def answer(
      day: Puzzle,
      part: String,
      input: String
  ): String < Puzzle.Effects =
    IO {
      answerInput(
        day,
        part,
        Input.fromStream(new ByteArrayInputStream(input.getBytes(UTF_8)))
      )
    }

  def checkSamples(day: Puzzle, part: String): Int < Puzzle.Effects =
    IO {
      require(
        Set("1", "2", "both").contains(part),
        "part must be 1, 2, or both"
      )
      val parts   = if (part == "both") Seq("1", "2") else Seq(part)
      val checks  = day.samples.zipWithIndex.flatMap { case (sample, index) =>
        Seq("1" -> sample.expected1, "2" -> sample.expected2).collect {
          case (number, Some(expected)) if parts.contains(number) =>
            (sample.input, index + 1, number, expected)
        }
      }
      val missing = parts.filterNot(number => checks.exists(_._3 == number))
      require(
        missing.isEmpty,
        s"No inline sample expectations for part(s) ${missing.mkString(", ")}; add a Sample before testing"
      )
      checks
    }.flatMap { checks =>
      Kyo
        .foreach(checks) { case (input, index, number, expected) =>
          answer(day, number, input).map { actual =>
            require(
              actual == expected,
              s"Sample $index part $number: expected $expected, got $actual"
            )
          }
        }
        .map(_.size)
    }

  def program(arguments: List[String]): Unit < Puzzle.Effects =
    IO {
      arguments match {
        case List("run", year, day, part)  =>
          for {
            solution <- IO(load(year, day))
            input    <- IO(Input.fromStream(java.lang.System.in))
            value    <- answerInput(solution, part, input)
            _        <- Console.printLine(value)
          } yield ()
        case List("test", year, day)       => program(List("test", year, day, "both"))
        case List("test", year, day, part) =>
          for {
            solution <- IO(load(year, day))
            checked  <- checkSamples(solution, part)
            _        <- Console.printLine(s"Passed $checked Scala sample assertion(s)")
          } yield ()
        case _                             =>
          throw IllegalArgumentException(
            "Usage: run YEAR DAY PART | test YEAR DAY [1|2|both]"
          )
      }
    }
}

object Main extends KyoApp {
  // Kyo's default error reporter prints to stdout. Keep stdout answer-only for
  // the CLI and benchmarks, and make both typed failures and panics fail fast.
  override protected def onResult(result: Result[Any, Any]): Unit =
    result match {
      case Result.Error(error: NotImplementedError) =>
        java.lang.System.err
          .println(s"Scala solution is an unfinished stub: ${error.getMessage}")
        exit(1)
      case Result.Error(error: Throwable)           =>
        java.lang.System.err.println(s"Scala: ${error.getMessage}")
        exit(1)
      case Result.Error(error)                      =>
        java.lang.System.err.println(s"Scala: $error")
        exit(1)
      case _                                        => ()
    }

  run(Abort.catching[Throwable](Runner.program(args.toList)))
}
