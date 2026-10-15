package land.yurian
package utils

import cats.Applicative
import cats.effect.IO
import cats.effect.Resource
import cats.syntax.all.*

import scala.io.Source
import cats.data.Kleisli
import cats.effect.kernel.Async
import fs2.io.file.Path
import fs2.Pipe
import fs2.io.file.Files
import scala.util.control.NoStackTrace
import cats.Eq
import cats.effect.std.Console
import cats.data.EitherT

sealed trait SolutionError                                            extends NoStackTrace
final case class IncorrectSampleAnswerError(star: "first" | "second") extends SolutionError
final case class ResourcePathError(throwable: Throwable)              extends SolutionError
final case class ReadingError(throwable: Throwable)                   extends SolutionError

object IOUtils:

    def getPath[F[_]: Async](fileName: String): EitherT[F, ResourcePathError, Path] =
        Async[F]
            .blocking(this.getClass.getClassLoader.getResource(fileName))
            .attemptT
            .leftMap(ResourcePathError(_))
            .map(_.getPath)
            .map(Path(_))

    def readFile(filePath: String): IO[Seq[String]] =
        IO(Source.fromFile(filePath)).bracket { source =>
            IO((for (line <- source.getLines) yield line).toVector)
        } { source =>
            IO(source.close())
        }

    private def getPathAndSolve[F[_]: Files: Async, A, O](
        prepare: Pipe[F, Byte, O],
        solve: fs2.Stream[F, O] => F[A],
    )(
        resource: String,
    ) =
        for
            resourcePath <- getPath(resource)
            result       <-
                solve(Files[F].readAll(resourcePath).through(prepare)).attemptT
                    .leftMap(ReadingError(_))
        yield result

    def solution[F[_]: Files: Async: Console, A: Eq, O](
        dayYear: String,
        sampleAnswer: A,
        prepare: Pipe[F, Byte, O] = identity,
        star: "first" | "second" = "first",
    )(
        solve: fs2.Stream[F, O] => F[A],
    ): EitherT[F, SolutionError, A] =
        val sampleResource = s"${dayYear}_sample_input.txt"
        val resource       = s"${dayYear}_input.txt"

        val compare: A => Either[IncorrectSampleAnswerError, Unit] = result =>
            if result === sampleAnswer then Right(())
            else Left(IncorrectSampleAnswerError(star))

        val compareAndPrint: A => EitherT[F, SolutionError, Unit] = result =>
            EitherT
                .fromEither(compare(result))
                .leftSemiflatTap: error =>
                    Console[F].errorln(s"Solution for ${error.star} star didn't match")
                .leftWiden

        val getPathAndSolveFor = getPathAndSolve(prepare, solve)

        for
            sampleResult <- getPathAndSolveFor(sampleResource)
            _            <- compareAndPrint(sampleResult)
            result       <- getPathAndSolveFor(resource)
            _            <- EitherT.right(Console[F].println(s"Result was: $result"))
        yield result
    end solution

    def solution[F[_]: Files: Async: Console, O, A: Eq, B: Eq](
        dayYear: String,
        firstSampleAnswer: A,
        secondSampleAnswer: B,
        prepare: Pipe[F, Byte, O],
    )(
        solveFirst: fs2.Stream[F, O] => F[A],
    )(
        solveSecond: fs2.Stream[F, O] => F[B],
    ): EitherT[F, SolutionError, (A, B)] =
        (
            solution(dayYear, firstSampleAnswer, prepare, "first")(solveFirst),
            solution(dayYear, secondSampleAnswer, prepare, "second")(solveSecond),
        ).tupled
end IOUtils
