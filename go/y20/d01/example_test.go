package d01

import "testing"

func TestExamples(t *testing.T) {
	t.Run("part1", func(t *testing.T) {
		if err := Solution.CheckExamples(1); err != nil {
			t.Fatal(err)
		}
	})
	t.Run("part2", func(t *testing.T) {
		if err := Solution.CheckExamples(2); err != nil {
			t.Fatal(err)
		}
	})
}
