// Package runner adapts ordinary solution functions to the shared CLI protocol.
package runner

type Key struct {
	Year int
	Day  int
}

type Day struct {
	Part1 func(string) any
	Part2 func(string) any
}

// Register permits each part to return its own idiomatic answer type.
func Register[A, B any](part1 func(string) A, part2 func(string) B) Day {
	return Day{
		Part1: func(input string) any { return part1(input) },
		Part2: func(input string) any { return part2(input) },
	}
}
