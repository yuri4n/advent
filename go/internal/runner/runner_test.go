package runner

import (
	"bytes"
	"fmt"
	"github.com/alecthomas/participle/v2"
	"strings"
	"testing"
)

type number struct {
	Value int `parser:"@Int"`
}

func TestAutomaticRunsSkipMissingPartsAndBenchmarksCopyMutableInput(t *testing.T) {
	day := New(func(input *Input) ([]int, error) { return []int{2, 3}, nil },
		(func([]int) (int, error))(nil),
		func(values []int) (int, error) { sum := values[0] + values[1]; values[0] = 0; return sum, nil },
		Example[int, int]{Input: "unused", Part2: Expect(5)})
	var out bytes.Buffer
	if err := day.Auto(Key{2026, 25}, t.TempDir(), &out); err != nil {
		t.Fatal(err)
	}
	if out.String() != "2026/25 example 1 part 2: 5\n" {
		t.Fatal(out.String())
	}
	report, err := day.Benchmark(2, "unused", 3, 1)
	if err != nil || report.Answer != "5" {
		t.Fatalf("%+v %v", report, err)
	}
}

func TestSharedProcessingIsLazyAndPreservesErrors(t *testing.T) {
	parser := participle.MustBuild[number]()
	calls := 0
	day := New(func(input *Input) (Stream[*number], error) {
		calls++
		return ParseLines(input.Lines(), parser), nil
	}, func(values Stream[*number]) (int, error) {
		value := 0
		var failure error
		values(func(n *number, err error) bool {
			failure = err
			if err == nil {
				value = n.Value
			}
			return false
		})
		return value, failure
	}, func(values Stream[*number]) (int, error) {
		all, err := Collect(values)
		total := 0
		for _, n := range all {
			total += n.Value
		}
		return total, err
	})
	for _, test := range []struct {
		part        int
		input, want string
	}{{1, "7\r\nbad\n", "7"}, {2, "7\n3\n", "10"}, {2, "7\n3\n", "10"}} {
		got, err := day.Run(test.part, strings.NewReader(test.input))
		if err != nil || fmt.Sprint(got) != test.want {
			t.Fatalf("got %v, %v", got, err)
		}
	}
	if calls != 3 {
		t.Fatalf("process called %d times", calls)
	}
	if _, err := day.Run(2, strings.NewReader("7\n3junk\n")); err == nil || !strings.Contains(err.Error(), "line 2") {
		t.Fatalf("missing located parse error: %v", err)
	}
	before := calls
	report, err := day.Benchmark(2, "7\n3\n", 2, 1)
	if err != nil || report.Answer != "10" || len(report.Timings) != 2 {
		t.Fatalf("benchmark: %+v, %v", report, err)
	}
	if calls-before != 4 {
		t.Fatalf("solve-only benchmark reprocessed input: %d calls", calls-before)
	}
}
