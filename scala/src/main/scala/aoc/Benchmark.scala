package aoc

import kyo.*

private[aoc] object Benchmark {
  final case class Report(
      answer: String,
      setupNanos: Long,
      timings: Vector[(Long, Long)]
  ) {
    def protocol: String =
      s"AOC_BENCH_V1\n$setupNanos\n" + timings
        .map((total, solve) => s"$total $solve\n")
        .mkString + answer + "\n"
  }

  private def measure[A](
      work: => A < Puzzle.Effects
  ): (A, Long) < Puzzle.Effects =
    Sync.defer(java.lang.System.nanoTime()).flatMap { start =>
      Scope.run(work).flatMap { value =>
        Sync.defer((value, java.lang.System.nanoTime() - start))
      }
    }

  def plan(
      day: Solution.WithAnswers,
      part: String,
      bytes: Array[Byte],
      iterations: Int,
      warmup: Int
  ): Report < Puzzle.Effects = Sync.defer {
    require(
      iterations >= 1 && iterations <= 10000 && warmup >= 0 && warmup <= 1000,
      "invalid benchmark counts"
    )
    require(day.availableParts.contains(part), s"part $part is unavailable")
    for {
      totals <- Kyo.foreach(0 until iterations + warmup)(_ =>
                  measure(day.answer(part, Input.fromBytes(bytes)))
                )
      setup  <-
        measure(day.process(Input.fromBytes(bytes)).flatMap(day.snapshot))
      solves <- Kyo.foreach(0 until iterations + warmup) { _ =>
                  // Fresh-copy/replay construction is outside the solve-only timer.
                  Sync
                    .defer(setup._1())
                    .flatMap(prepared =>
                      measure(day.preparedAnswer(part, prepared))
                    )
                }
      report <- Sync.defer {
                  val expected = totals.head._1
                  require(
                    (totals ++ solves).forall(_._1 == expected),
                    "answer changed between benchmark runs or modes"
                  )
                  Report(
                    expected,
                    setup._2,
                    totals
                      .drop(warmup)
                      .zip(solves.drop(warmup))
                      .map((a, b) => (a._2, b._2))
                      .toVector
                  )
                }
    } yield report
  }
}
