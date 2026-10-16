package aoc

import kyo.*
import kyo.test.AssertScope

/** Common assertions/registration for the native Kyo test runner. */
abstract class PuzzleTest extends kyo.test.Test[Any] {
  protected inline def test(inline name: String)(
      inline body: AssertScope ?=> Any < (Async & Abort[Any] & Scope)
  )(using inline frame: Frame): Unit = name in body.unit

  protected inline def assertEquals[A](inline actual: A, inline expected: A)(
      using
      inline scope: AssertScope,
      inline frame: Frame
  ): Unit = {
    given CanEqual[A, A] = CanEqual.derived
    assert(actual == expected)
  }
}
