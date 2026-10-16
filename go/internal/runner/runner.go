// Package runner provides a typed, stream-first Advent of Code contract.
package runner

import (
	"fmt"
	"io"
	"strings"

	"github.com/alecthomas/participle/v2"
)

type Key struct{ Year, Day int }

// Stream is a single-pass push iterator. Return false to stop reading early.
// This spelling also works on Go 1.19, without requiring range-over-function.
type Stream[T any] func(yield func(T, error) bool)

type Example[A, B any] struct {
	Input string
	Part1 *A
	Part2 *B
}

func Expect[A any](answer A) *A { return &answer }

type storedExample struct {
	Input        string
	Part1, Part2 *string
}
type Day struct {
	parts    [2]func(io.Reader) (any, error)
	examples []storedExample
	bench    func(int, string, int, int) (Benchmark, error)
}

// New shares process between both parts. P may be a stream or a collection.
func New[P, A, B any](process func(*Input) (P, error), part1 func(P) (A, error), part2 func(P) (B, error), examples ...Example[A, B]) Day {
	day := Day{}
	if part1 != nil {
		day.parts[0] = func(reader io.Reader) (any, error) {
			input, err := process(NewInput(reader))
			if err != nil {
				return nil, err
			}
			return part1(input)
		}
	}
	if part2 != nil {
		day.parts[1] = func(reader io.Reader) (any, error) {
			input, err := process(NewInput(reader))
			if err != nil {
				return nil, err
			}
			return part2(input)
		}
	}
	for _, example := range examples {
		stored := storedExample{Input: example.Input}
		if example.Part1 != nil {
			stored.Part1 = Expect(fmt.Sprint(*example.Part1))
		}
		if example.Part2 != nil {
			stored.Part2 = Expect(fmt.Sprint(*example.Part2))
		}
		day.examples = append(day.examples, stored)
	}
	day.bench = func(part int, text string, iterations, warmup int) (Benchmark, error) {
		return benchmark(text, iterations, warmup, process, func(input P) (any, error) {
			if part == 1 {
				return part1(input)
			}
			return part2(input)
		})
	}
	return day
}

// Register preserves the old whole-text API for external/in-progress solutions.
func Register[A, B any](part1 func(string) A, part2 func(string) B) Day {
	return Day{parts: [2]func(io.Reader) (any, error){
		func(reader io.Reader) (any, error) {
			data, err := io.ReadAll(reader)
			if err != nil {
				return nil, err
			}
			return part1(string(data)), nil
		},
		func(reader io.Reader) (any, error) {
			data, err := io.ReadAll(reader)
			if err != nil {
				return nil, err
			}
			return part2(string(data)), nil
		},
	}}
}

func (day Day) Run(part int, reader io.Reader) (any, error) {
	if part != 1 && part != 2 {
		return nil, fmt.Errorf("part must be 1 or 2")
	}
	if day.parts[part-1] == nil {
		return nil, fmt.Errorf("part %d is unavailable", part)
	}
	return day.parts[part-1](reader)
}

func (day Day) Example(index int) (string, error) {
	if index < 1 || index > len(day.examples) {
		return "", fmt.Errorf("example %d does not exist; this day declares %d example(s)", index, len(day.examples))
	}
	return day.examples[index-1].Input, nil
}

func (day Day) Expectation(part, index int) (*string, error) {
	if _, err := day.Example(index); err != nil {
		return nil, err
	}
	switch part {
	case 1:
		return day.examples[index-1].Part1, nil
	case 2:
		return day.examples[index-1].Part2, nil
	default:
		return nil, fmt.Errorf("part must be 1 or 2")
	}
}

func (day Day) CheckExamples(part int) (failure error) {
	defer func() {
		if problem := recover(); problem != nil {
			failure = fmt.Errorf("solution failed: %v", problem)
		}
	}()
	if part != 1 && part != 2 {
		return fmt.Errorf("part must be 1 or 2")
	}
	checked := 0
	for index, example := range day.examples {
		expected := example.Part1
		if part == 2 {
			expected = example.Part2
		}
		if expected == nil {
			continue
		}
		actual, err := day.Run(part, strings.NewReader(example.Input))
		if err != nil {
			return fmt.Errorf("example %d part %d: %w", index+1, part, err)
		}
		if fmt.Sprint(actual) != *expected {
			return fmt.Errorf("example %d part %d: expected %s, got %v", index+1, part, *expected, actual)
		}
		checked++
	}
	if checked == 0 {
		return fmt.Errorf("no examples: add an expectation for part %d", part)
	}
	return nil
}

// Lines is the compatibility helper for a borrowed reader, with no record-size limit.
func Lines(source io.Reader) Stream[string] { return NewInput(source).Lines() }

// ParseLines applies a Participle grammar lazily; trailing tokens are errors.
func ParseLines[T any](lines Stream[string], parser *participle.Parser[T]) Stream[*T] {
	return func(yield func(*T, error) bool) {
		index := 0
		lines(func(line string, err error) bool {
			index++
			if err != nil {
				yield(nil, fmt.Errorf("line %d: %w", index, err))
				return false
			}
			value, err := parser.ParseString(fmt.Sprintf("line %d", index), line)
			keepGoing := yield(value, err)
			return keepGoing && err == nil
		})
	}
}

func Collect[T any](stream Stream[T]) ([]T, error) {
	var values []T
	var failure error
	stream(func(value T, err error) bool {
		if err != nil {
			failure = err
			return false
		}
		values = append(values, value)
		return true
	})
	return values, failure
}
