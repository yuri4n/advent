package aoc

import java.io.{BufferedReader, InputStream, InputStreamReader}
import java.nio.charset.StandardCharsets.UTF_8
import java.util.concurrent.atomic.AtomicBoolean
import kyo.*

/** A borrowed input, consumed once as text, lines, or bytes. The runner
  * supplies a fresh instance for each part and sample and owns the underlying
  * stream.
  */
final class Input private (source: InputStream) {
  private val consumed = new AtomicBoolean(false)

  private def claim(): Unit =
    if (!consumed.compareAndSet(false, true))
      throw IllegalStateException(
        "Input has already been consumed; use a fresh input for each part or representation"
      )

  /** Read the complete input as UTF-8 text, preserving line endings. */
  def text: String < IO = IO {
    claim()
    new String(source.readAllBytes(), UTF_8)
  }

  /** Stream UTF-8 lines without their terminators. Reader buffering may read
    * ahead; memory is bounded by those buffers and the longest line.
    */
  def lines: Stream[String, IO] = Stream {
    IO {
      claim()
      new BufferedReader(new InputStreamReader(source, UTF_8), 4096)
    }.flatMap { reader =>
      Stream
        .repeatPresent[String, IO](
          IO {
            val line = reader.readLine()
            if (line == null) Absent else Present(Seq(line))
          },
          chunkSize = 1
        )
        .emit
    }
  }

  /** Stream unmodified bytes in bounded 4 KiB chunks. */
  def bytes: Stream[Byte, IO] = Stream {
    IO {
      claim()
      new Array[Byte](4096)
    }.flatMap { buffer =>
      Stream
        .repeatPresent[Byte, IO](
          IO {
            val size = source.read(buffer)
            if (size < 0) Absent else Present(buffer.take(size).toIndexedSeq)
          },
          chunkSize = 4096
        )
        .emit
    }
  }
}

object Input {
  private[aoc] def fromStream(source: InputStream): Input = new Input(source)
}
