package main

import (
	"fmt"
	"io"
	"os"
	"sort"
	"strconv"

	"github.com/yurianthedev/advent/internal/runner"
)

func run(args []string, input io.Reader, output io.Writer) error {
	if len(args) == 0 || args[0] == "auto" {
		keys := make([]runner.Key, 0, len(days))
		if len(args) == 3 {
			year, err := strconv.Atoi(args[1])
			if err != nil {
				return err
			}
			day, err := strconv.Atoi(args[2])
			if err != nil {
				return err
			}
			key := runner.Key{Year: year, Day: day}
			if _, ok := days[key]; !ok {
				return fmt.Errorf("no Go solution for %d day %d", year, day)
			}
			keys = append(keys, key)
		} else if len(args) <= 1 {
			for key := range days {
				keys = append(keys, key)
			}
		} else {
			return fmt.Errorf("usage: auto [YEAR DAY]")
		}
		sort.Slice(keys, func(i, j int) bool {
			if keys[i].Year == keys[j].Year {
				return keys[i].Day < keys[j].Day
			}
			return keys[i].Year < keys[j].Year
		})
		for _, key := range keys {
			if err := days[key].Auto(key, runner.Root(), output); err != nil {
				return err
			}
		}
		return nil
	}
	mode := "run"
	if args[0] == "example" || args[0] == "bench" || args[0] == "expect" {
		mode = args[0]
		args = args[1:]
	}
	count := 3
	if mode == "bench" {
		count = 5
	} else if mode == "expect" {
		count = 4
	}
	if len(args) != count {
		return fmt.Errorf("usage: YEAR DAY PART | example YEAR DAY INDEX | bench YEAR DAY PART ITERATIONS WARMUP | auto [YEAR DAY]")
	}
	values := make([]int, count)
	for i, arg := range args {
		value, err := strconv.Atoi(arg)
		if err != nil {
			return fmt.Errorf("invalid integer %q", arg)
		}
		values[i] = value
	}
	day, found := days[runner.Key{Year: values[0], Day: values[1]}]
	if !found {
		return fmt.Errorf("no Go solution for %d day %d", values[0], values[1])
	}
	if mode == "expect" {
		expected, err := day.Expectation(values[2], values[3])
		if err != nil {
			return err
		}
		if expected == nil {
			_, err = fmt.Fprintln(output, "AOC_NO_EXPECT_V1")
		} else {
			_, err = fmt.Fprintf(output, "AOC_EXPECT_V1\n%s\n", *expected)
		}
		return err
	}
	if mode == "example" {
		text, err := day.Example(values[2])
		if err != nil {
			return err
		}
		_, err = io.WriteString(output, text)
		return err
	}
	if values[2] != 1 && values[2] != 2 {
		return fmt.Errorf("part must be 1 or 2")
	}
	if mode == "bench" {
		text, err := io.ReadAll(input)
		if err != nil {
			return err
		}
		report, err := day.Benchmark(values[2], string(text), values[3], values[4])
		if err != nil {
			return err
		}
		return report.Print(output)
	}
	answer, err := day.Run(values[2], input)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintln(output, answer)
	return err
}

func main() {
	defer func() {
		if problem := recover(); problem != nil {
			fmt.Fprintln(os.Stderr, "Go solution:", problem)
			os.Exit(1)
		}
	}()
	if err := run(os.Args[1:], os.Stdin, os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
