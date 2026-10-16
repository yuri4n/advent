package aoc

import java.io.{BufferedReader, InputStream, InputStreamReader, PushbackReader}
import java.nio.ByteBuffer
import java.nio.charset.CodingErrorAction.REPORT
import java.nio.charset.StandardCharsets.UTF_8
import java.util.concurrent.atomic.AtomicBoolean
import kyo.*

/** A borrowed byte source with single-pass text, character, line and byte
  * views. The runner supplies a fresh instance for each part and sample and
  * owns the underlying stream.
  */
final class Input private (source: InputStream) {
  private val consumed = new AtomicBoolean(false)

  private def claim(): Unit =
    if (!consumed.compareAndSet(false, true))
      throw IllegalStateException(
        "Input has already been consumed; use a fresh input for each part or representation"
      )

  /** Read the complete input as UTF-8 text, preserving line endings. */
  def text: String < Sync = Sync.defer {
    claim()
    UTF_8
      .newDecoder()
      .onMalformedInput(REPORT)
      .onUnmappableCharacter(REPORT)
      .decode(ByteBuffer.wrap(source.readAllBytes()))
      .toString
  }

  /** Stream UTF-8 lines without their terminators. Reader buffering may read
    * ahead; memory is bounded by those buffers and the longest line.
    */
  def lines: Stream[String, Sync] = Stream {
    Sync
      .defer {
        claim()
        textReader()
      }
      .flatMap { reader =>
        Stream
          .repeatPresent[String, Sync](
            Sync.defer {
              val line = reader.readLine()
              if (line == null) Absent else Present(Seq(line))
            },
            chunkSize = 1
          )
          .emit
      }
  }

  private def textReader(): BufferedReader =
    new BufferedReader(
      new InputStreamReader(
        source,
        UTF_8
          .newDecoder()
          .onMalformedInput(REPORT)
          .onUnmappableCharacter(REPORT)
      ),
      4096
    )

  /** Native Scala characters (UTF-16 code units), decoded incrementally. */
  def chars: Stream[Char, Sync] = Stream {
    Sync.defer { claim(); textReader() }.flatMap { reader =>
      val buffer = new Array[Char](4096)
      Stream
        .repeatPresent[Char, Sync](Sync.defer {
          val size = reader.read(buffer)
          if (size < 0) Absent else Present(buffer.take(size).toIndexedSeq)
        })
        .emit
    }
  }

  /** Lines whose characters are read on demand, without buffering a whole line.
    * Advancing the outer stream drains the unread current line. Old line views
    * then expire; consume each inner stream before advancing the outer stream.
    */
  def lineChars: Stream[Stream[Char, Sync], Sync] = Stream {
    Sync
      .defer {
        claim(); new Input.LineCursor(new PushbackReader(textReader(), 1))
      }
      .flatMap { cursor =>
        Stream
          .repeatPresent[Stream[Char, Sync], Sync](
            Sync.defer {
              cursor.nextLine() match {
                case Some(line) => Present(Seq(line))
                case None       => Absent
              }
            },
            chunkSize = 1
          )
          .emit
      }
  }

  /** Stream unmodified bytes in bounded 4 KiB chunks. */
  def bytes: Stream[Byte, Sync] = Stream {
    Sync
      .defer {
        claim()
        new Array[Byte](4096)
      }
      .flatMap { buffer =>
        Stream
          .repeatPresent[Byte, Sync](
            Sync.defer {
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
  private class LineCursor(reader: PushbackReader) {
    private var generation                     = 0L
    private var ended                          = true
    private def peek(): Int                    = {
      val c = reader.read()
      if (c >= 0) reader.unread(c)
      c
    }
    private def nextChar(id: Long): Int        = {
      require(
        id == generation,
        "line view expired; consume it before advancing to the next line"
      )
      if (ended) -1
      else {
        val c = reader.read()
        if (c < 0 || c == '\n' || c == '\r') {
          if (c == '\r' && peek() == '\n') { val _ = reader.read() }
          ended = true
          -1
        } else c
      }
    }
    def nextLine(): Option[Stream[Char, Sync]] = {
      while (!ended) { val _ = nextChar(generation) }
      generation += 1
      if (peek() < 0) None
      else {
        ended = false
        val id   = generation
        val used = new AtomicBoolean(false)
        Some(Stream {
          Sync
            .defer {
              require(
                used.compareAndSet(false, true),
                "line stream already consumed"
              )
            }
            .andThen(
              Stream
                .repeatPresent[Char, Sync](
                  Sync.defer {
                    val c = nextChar(id)
                    if (c < 0) Absent else Present(Seq(c.toChar))
                  },
                  chunkSize = 1
                )
                .emit
            )
        })
      }
    }
  }

  def fromText(text: String): Input          = fromBytes(text.getBytes(UTF_8))
  def fromBytes(bytes: Array[Byte]): Input   =
    fromStream(new java.io.ByteArrayInputStream(bytes))
  def fromStream(source: InputStream): Input = new Input(source)
}
