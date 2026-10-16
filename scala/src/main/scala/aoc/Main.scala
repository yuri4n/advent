package aoc

import java.nio.file.{Files, Path, Paths}
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
    Sync.defer {
      require(part == "1" || part == "2", "part must be 1 or 2")
      day.answer(part, input)
    }

  def answer(
      day: Puzzle,
      part: String,
      input: String
  ): String < Puzzle.Effects =
    Sync.defer {
      answerInput(
        day,
        part,
        Input.fromText(input)
      )
    }

  def defaultInput(day: Puzzle): Option[Path] = {
    val name = "aoc\\.solutions\\.y([0-9]{4})\\.Day([0-9]{2})\\$".r
    day.getClass.getName match {
      case name(year, number) =>
        val root = Option(java.lang.System.getenv("AOC_ROOT"))
          .filter(_.nonEmpty)
          .map(Paths.get(_))
          .orElse {
            Iterator
              .iterate(Option(Paths.get("").toAbsolutePath))(
                _.flatMap(path => Option(path.getParent))
              )
              .takeWhile(_.nonEmpty)
              .flatten
              .find(path => Files.isRegularFile(path.resolve("aoc.toml")))
          }
        root.map(
          _.resolve(
            f"data/y${year.toInt % 100}%02d/d${number.toInt}%02d/input.txt"
          )
        )
      case _                  => None
    }
  }

  def auto(
      day: Puzzle,
      path: Option[Path]
  ): Vector[(String, String)] < Puzzle.Effects = {
    val cases = for {
      (example, index) <- day.examples.zipWithIndex
      part             <- day.availableParts
      expected          = if (part == "1") example.expected1 else example.expected2
      if expected.nonEmpty || (example.expected1.isEmpty && example.expected2.isEmpty)
    } yield (example, index + 1, part, expected)
    for {
      examples <- Kyo.foreach(cases) { case (example, index, part, expected) =>
                    Scope
                      .run(day.runExample(part, index - 1))
                      .map(value => (s"Example $index part $part", value))
                  }
      real     <- Kyo.foreach(
                    path
                      .filter(Files.isRegularFile(_))
                      .toSeq
                      .flatMap(p => day.availableParts.map(p -> _))
                  ) { case (file, part) =>
                    Scope.run {
                      Sync.defer(Files.newInputStream(file)).flatMap { stream =>
                        Scope.ensure(Sync.defer(stream.close())).andThen {
                          answerInput(day, part, Input.fromStream(stream))
                            .map(value => (s"Input part $part", value))
                        }
                      }
                    }
                  }
    } yield (examples ++ real).toVector
  }

  def exampleInput(day: Puzzle, index: Int): String = {
    require(
      index >= 1 && index <= day.examples.size,
      s"Example $index does not exist; this day declares ${day.examples.size} example(s)"
    )
    day.examples(index - 1).input
  }

  def checkSamples(day: Puzzle, part: String): Int < Puzzle.Effects =
    checkExamples(day, part)

  def checkExamples(day: Puzzle, part: String): Int < Puzzle.Effects =
    Sync
      .defer {
        require(
          Set("1", "2", "both").contains(part),
          "part must be 1, 2, or both"
        )
        val parts   = if (part == "both") Seq("1", "2") else Seq(part)
        val checks  = day.examples.zipWithIndex.flatMap { case (sample, index) =>
          Seq("1" -> sample.expected1, "2" -> sample.expected2).collect {
            case (number, Some(expected)) if parts.contains(number) =>
              (sample.input, index + 1, number, expected)
          }
        }
        val missing = parts.filterNot(number => checks.exists(_._3 == number))
        require(
          missing.isEmpty,
          s"No inline example expectations for part(s) ${missing.mkString(", ")}; add an Example before testing"
        )
        checks
      }
      .flatMap { checks =>
        Kyo
          .foreach(checks) { case (_, index, number, _) =>
            day.runExample(number, index - 1).unit
          }
          .map(_.size)
      }

  def program(arguments: List[String]): Unit < Puzzle.Effects =
    Sync.defer {
      arguments match {
        case List("run", year, day, part)                       =>
          for {
            solution <- Sync.defer(load(year, day))
            input    <- Sync.defer(Input.fromStream(java.lang.System.in))
            value    <- answerInput(solution, part, input)
            _        <- Console.printLine(value)
          } yield ()
        case List("bench", year, day, part, iterations, warmup) =>
          Sync.defer(load(year, day)).flatMap {
            case solution: Solution.WithAnswers =>
              Sync
                .defer(java.lang.System.in.readAllBytes())
                .flatMap { bytes =>
                  solution
                    .benchmarkBytes(part, bytes, iterations.toInt, warmup.toInt)
                    .flatMap(report =>
                      Sync.defer(java.lang.System.out.print(report.protocol))
                    )
                }
            case _                              =>
              throw IllegalArgumentException("legacy day: use aoc bench --wall")
          }
        case List("expect", year, day, part, index)             =>
          Sync.defer {
            require(part == "1" || part == "2", "part must be 1 or 2")
            val solution = load(year, day)
            val _        = exampleInput(solution, index.toInt)
            val example  = solution.examples(index.toInt - 1)
            val expected =
              if (part == "1") example.expected1 else example.expected2
            java.lang.System.out.print(
              expected.fold("AOC_NO_EXPECT_V1\n")(value =>
                s"AOC_EXPECT_V1\n$value\n"
              )
            )
          }
        case List("example", year, day, index)                  =>
          Sync.defer {
            java.lang.System.out
              .print(exampleInput(load(year, day), index.toInt))
          }
        case List("test", year, day)                            => program(List("test", year, day, "both"))
        case List("test", year, day, part)                      =>
          for {
            solution <- Sync.defer(load(year, day))
            checked  <- checkExamples(solution, part)
            _        <-
              Console.printLine(s"Passed $checked Scala example assertion(s)")
          } yield ()
        case _                                                  =>
          throw IllegalArgumentException(
            "Usage: run YEAR DAY PART | test YEAR DAY [1|2|both] | example YEAR DAY INDEX"
          )
      }
    }
}

// RC7 routes failures to stderr and exits nonzero; successful Unit is silent.
abstract class PuzzleApp extends KyoApp

object Main extends PuzzleApp {
  run(Abort.catching[Throwable](Runner.program(args.toList)))
}
