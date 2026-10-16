package aoc

import kyo.*

/** Small missing primitives; compose them with Kyo's native Parse combinators.
  */
object Parsing {

  /** One or more ASCII digits, bounded by Long.MaxValue. Stops before the first
    * non-digit, accepts zero, and rejects signs/overflow as parse failures. Use
    * Parse.entireInput around the enclosing grammar when appropriate.
    */
  def nonNegativeLong(using Frame): Long < Parse[Char] =
    Parse.read { input =>
      val digits = input.remaining.takeWhile(c => c >= '0' && c <= '9')
      digits.mkString.toLongOption match {
        case Some(value) =>
          Result.succeed((input.advance(digits.length), value))
        case None        =>
          Result.fail(
            Chunk(
              ParseFailure(
                "Expected a non-negative decimal Long (0..Long.MaxValue)",
                input.position
              )
            )
          )
      }
    }
}
