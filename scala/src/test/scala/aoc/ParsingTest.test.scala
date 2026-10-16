package aoc

import kyo.*

class ParsingTest extends PuzzleTest {
  test(
    "nonNegativeLong preserves delimiters and checks the complete Long range"
  ) {
    val range = Parse
      .inOrder(
        Parsing.nonNegativeLong,
        Parse.literal("-"),
        Parsing.nonNegativeLong
      )
      .map((first, _, last) => (first, last))
    for {
      pair <- Parse.runOrAbort("11-22")(Parse.entireInput(range))
      _     = assertEquals(pair, (11L, 22L))
      _    <- Kyo.foreach(
                Seq(
                  "0"                    -> 0L,
                  "0007"                 -> 7L,
                  "2147483648"           -> 2147483648L,
                  Long.MaxValue.toString -> Long.MaxValue
                )
              ) { case (text, expected) =>
                Parse
                  .runOrAbort(text)(Parse.entireInput(Parsing.nonNegativeLong))
                  .map(actual => assertEquals(actual, expected))
              }
      _    <- Kyo.foreach(
                Seq(
                  "",
                  "-1",
                  "+1",
                  " 1",
                  "١",
                  "9223372036854775808",
                  "999999999999999999999999",
                  "12junk"
                )
              ) { text =>
                Abort
                  .run[ParseError](
                    Parse.runOrAbort(text)(
                      Parse.entireInput(Parsing.nonNegativeLong)
                    )
                  )
                  .map(result => assert(result.isFailure, text))
              }
    } yield ()
  }
}
