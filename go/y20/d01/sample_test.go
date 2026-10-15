package d01

import "testing"

const sample = `1721
979
366
299
675
1456
`

func TestSamples(t *testing.T) {
	t.Run("part1", func(t *testing.T) {
		if got := Part1(sample); got != 514579 {
			t.Fatalf("Part1(sample) = %d, want 514579", got)
		}
	})
	t.Run("part2", func(t *testing.T) {
		if got := Part2(sample); got != 241861950 {
			t.Fatalf("Part2(sample) = %d, want 241861950", got)
		}
	})
}
