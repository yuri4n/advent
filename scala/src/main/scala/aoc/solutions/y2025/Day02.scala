package aoc.solutions.y2025

import aoc.{Example, Example1, Input, Solution}
import kyo.*

object Day02 extends Solution {
  type Prepared = Stream[String, Sync]
  type Answer1 = Long

  def process(input: Input): Prepared < Any = input.lines

  // Register a part only after implementing it:
  // override val part1 = solve { (input: Prepared) => ... }
  // override val part2 = solve { (input: Prepared) => ... }

  // Add Example1("input", expectedAnswer) as you implement a part.
  override val examples: Seq[Example[Answer1, Answer2]] = Seq.empty
}
