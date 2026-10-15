package aoc.solutions.y2021

import aoc.{Day, Sample}
import kyo.<

object Day03 extends Day[Int, Int] {
  type Binary = Seq[Byte]

  def parse(input: String): Seq[Binary] =
    input.linesIterator.map(_.map(_.asDigit.toByte).toSeq).toVector

  def part1(input: String): Int < Any = {
    val gamma = genGamma(parse(input).transpose)
    binaryToInt(gamma) * binaryToInt(genEpsilon(gamma))
  }

  def part2(input: String): Int < Any = {
    val binaries      = parse(input)
    val processedList = binaries.transpose
    binaryToInt(genOxygenRating(processedList, binaries)) *
      binaryToInt(genCO2Rating(processedList, binaries))
  }

  def binaryToInt(binary: Binary): Int = Integer.parseInt(binary.mkString, 2)

  def mostRepeated(list: Seq[Byte]): Byte = {
    val values    = list.groupBy(identity).values
    val valuesLen = values.map(_.length)
    if (valuesLen.forall(_ == valuesLen.head)) 1.toByte
    else values.maxBy(_.length).head
  }

  def inverseByte(byte: Byte): Byte = byte match {
    case 1 => 0
    case 0 => 1
  }

  def genGamma(processedList: Seq[Seq[Byte]]): Binary =
    processedList.map(mostRepeated)
  def genEpsilon(gamma: Binary): Binary               = gamma.map(inverseByte)

  def genRating(
      transposed: Seq[Seq[Byte]],
      binaries: Seq[Binary],
      transformation: Byte => Byte
  ): Binary =
    transposed
      .foldLeft((0, binaries)) { case ((pos, candidates), _) =>
        (
          pos + 1, {
            val filteredCandidates = candidates.filter { b =>
              val ct = candidates.transpose
              b(pos) == transformation(mostRepeated(ct(pos)))
            }
            if (candidates.sizeIs > 1 && filteredCandidates.sizeIs > 0)
              filteredCandidates
            else candidates
          }
        )
      }
      ._2
      .head

  def genOxygenRating(
      transposed: Seq[Seq[Byte]],
      binaries: Seq[Binary]
  ): Binary =
    genRating(transposed, binaries, identity)

  def genCO2Rating(transposed: Seq[Seq[Byte]], binaries: Seq[Binary]): Binary =
    genRating(transposed, binaries, inverseByte)

  override val samples = Seq(
    Sample(
      "00100\n11110\n10110\n10111\n10101\n01111\n00111\n11100\n10000\n11001\n00010\n01010\n",
      Some("198"),
      Some("230")
    )
  )
}
