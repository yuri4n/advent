package aoc

import kyo.*

/** Inferred when a part is registered, while Prepared's element/effect types
  * are still known. In particular, Kyo's Emit tag must not be erased to Any.
  */
private[aoc] trait Replay[A] {
  def apply(input: A): (() => A) < Puzzle.Effects
}

private[aoc] object Replay extends ValueReplay {
  given stream[A, S](using
      tag: Tag[Emit[Chunk[A]]],
      effects: Stream[A, S] <:< Stream[A, Puzzle.Effects],
      item: Replay[A]
  ): Replay[Stream[A, S]] with {
    def apply(input: Stream[A, S]): (() => Stream[A, S]) < Puzzle.Effects =
      effects(input)
        .fold(Vector.empty[() => A])((saved, value) =>
          item(value).map(saved :+ _)
        )
        .map(saved => () => Stream.init(saved.map(_())))
  }
}

private[aoc] trait ValueReplay {
  given value[A]: Replay[A] with {
    def apply(input: A): (() => A) < Puzzle.Effects =
      input match {
        case _: Stream[?, ?] =>
          throw UnsupportedOperationException(
            "stream effects need a custom benchmark snapshot; override snapshot or use --wall"
          )
        case _: Iterator[?] | _: scala.collection.View[?] |
            _: scala.collection.immutable.LazyList[?] =>
          throw UnsupportedOperationException(
            "lazy prepared input needs a materializing benchmark replay; override snapshot or use --wall"
          )
        case _: Array[?] | _: scala.collection.mutable.Iterable[?] |
            _: java.util.Collection[?] | _: java.util.Map[?, ?] =>
          throw UnsupportedOperationException(
            "mutable prepared input needs a copying benchmark replay; override snapshot or use --wall"
          )
        case _               => () => input
      }
  }
}
