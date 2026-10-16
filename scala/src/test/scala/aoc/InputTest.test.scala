package aoc

import java.io.{ByteArrayInputStream, IOException, InputStream}
import java.nio.charset.StandardCharsets.UTF_8
import kyo.*

class InputTest extends PuzzleTest {
  private def run[A](value: => A < Sync): A = {
    import AllowUnsafe.embrace.danger
    KyoApp.Unsafe
      .runAndBlock(5.seconds)(Abort.catching[Throwable](value))
      .getOrThrow
  }

  private class TrackedSource(data: Array[Byte])
      extends ByteArrayInputStream(data) {
    var readBytes                                                         = 0
    var closed                                                            = false
    override def read(buffer: Array[Byte], offset: Int, length: Int): Int = {
      val count = super.read(buffer, offset, length)
      if (count > 0) readBytes += count
      count
    }
    override def close(): Unit                                            = { closed = true }
  }

  private def fromText(text: String): Input =
    Input.fromStream(new ByteArrayInputStream(text.getBytes(UTF_8)))

  test("creating input views neither reads nor claims the source") {
    val source     = new TrackedSource("hello\n".getBytes(UTF_8))
    val input      = Input.fromStream(source)
    val text       = input.text
    val lines      = input.lines
    val bytes      = input.bytes
    assertEquals(source.readBytes, 0)
    assertEquals(run(lines.fold(Vector.empty[String])(_ :+ _)), Vector("hello"))
    val textError  = intercept[IllegalStateException](run(text))
    val bytesError =
      intercept[IllegalStateException](run(bytes.fold(0)((n, _) => n + 1)))
    assert(textError.getMessage.contains("already been consumed"))
    assert(bytesError.getMessage.contains("already been consumed"))
    assert(!source.closed)
  }

  test("text preserves UTF-8 and original line endings") {
    val text = "café\r\n雪\n\nlast"
    assertEquals(run(fromText(text).text), text)
    assertEquals(run(fromText("").text), "")
  }

  test(
    "lines decode UTF-8 and handle empty input, blank lines, CRLF, and a final line"
  ) {
    val cases = Seq(
      ""                -> Vector.empty[String],
      "\n"              -> Vector(""),
      "\n\n"            -> Vector("", ""),
      "a\r\nb\r\n"      -> Vector("a", "b"),
      "a\rb\r"          -> Vector("a", "b"),
      "café\n雪\n\nlast" -> Vector("café", "雪", "", "last")
    )
    for ((text, expected) <- cases)
      assertEquals(
        run(fromText(text).lines.fold(Vector.empty[String])(_ :+ _)),
        expected
      )
  }

  test(
    "character views decode UTF-8 and nested lines do not collect whole lines"
  ) {
    val text   = "é雪🙂\r\n\nlast"
    assertEquals(run(fromText(text).chars.fold("")(_ + _)), text)
    assertEquals(
      run(
        fromText(text).lineChars
          .map(_.fold("")(_ + _))
          .fold(Vector.empty[String])(_ :+ _)
      ),
      Vector("é雪🙂", "", "last")
    )
    val source =
      new TrackedSource(("a" * 100000 + "\nsecond\n").getBytes(UTF_8))
    assertEquals(
      run(
        Input
          .fromStream(source)
          .lineChars
          .take(1)
          .map(_.take(2).fold("")(_ + _))
          .fold(Vector.empty[String])(_ :+ _)
      ),
      Vector("aa")
    )
    assert(
      source.readBytes <= 8192,
      "a nested line must not be buffered in full"
    )
    assertEquals(
      run(
        fromText("abcdef\r\nxyz\n").lineChars
          .map(_.take(2).fold("")(_ + _))
          .fold(Vector.empty[String])(_ :+ _)
      ),
      Vector("ab", "xy")
    )
  }

  test("bytes preserve arbitrary binary data across chunk boundaries") {
    val data   = Array.tabulate[Byte](10003)(index => (index % 256).toByte)
    val source = new TrackedSource(data)
    val actual =
      run(Input.fromStream(source).bytes.fold(Vector.empty[Byte])(_ :+ _))
    assertEquals(actual, data.toVector)
    assertEquals(source.readBytes, data.length)
    assert(!source.closed)
    assertEquals(
      run(
        Input
          .fromStream(new ByteArrayInputStream(Array.emptyByteArray))
          .bytes
          .fold(0)((n, _) => n + 1)
      ),
      0
    )
  }

  test(
    "taking one byte reads only a bounded chunk and keeps borrowed input open"
  ) {
    val source = new TrackedSource(new Array[Byte](100000))
    val count  =
      run(Input.fromStream(source).bytes.take(1).fold(0)((n, _) => n + 1))
    assertEquals(count, 1)
    assert(source.readBytes > 0 && source.readBytes <= 4096)
    assert(!source.closed)
  }

  test(
    "taking one line permits bounded reader read-ahead without draining the source"
  ) {
    val source = new TrackedSource(("line\n" * 20000).getBytes(UTF_8))
    val first  = run(
      Input.fromStream(source).lines.take(1).fold(Vector.empty[String])(_ :+ _)
    )
    assertEquals(first, Vector("line"))
    assert(source.readBytes > 0 && source.readBytes <= 8192)
    assert(!source.closed)
  }

  test("replaying a stream or mixing representations fails clearly") {
    val input       = fromText("one\ntwo\n")
    val lines       = input.lines
    assertEquals(run(lines.take(1).fold(0)((n, _) => n + 1)), 1)
    val replayError =
      intercept[IllegalStateException](run(lines.fold(0)((n, _) => n + 1)))
    assert(replayError.getMessage.contains("fresh input"))
    val textInput   = fromText("one")
    assertEquals(run(textInput.text), "one")
    val mixedError  = intercept[IllegalStateException](
      run(textInput.lines.fold(0)((n, _) => n + 1))
    )
    assert(mixedError.getMessage.contains("fresh input"))
  }

  test(
    "read errors propagate without closing or permitting a second consumption"
  ) {
    var closed     = false
    val failure    = IOException("broken input")
    val source     = new InputStream {
      def read(): Int            = throw failure
      override def close(): Unit = { closed = true }
    }
    val input      = Input.fromStream(source)
    val caught     =
      intercept[IOException](run(input.bytes.fold(0)((n, _) => n + 1)))
    assert(caught eq failure)
    assert(!closed)
    val reuseError = intercept[IllegalStateException](run(input.text))
    assert(reuseError.getMessage.contains("already been consumed"))
  }
}
