package main

import (
	"fmt"
	"io"
	"os"
	"strconv"

	"github.com/yurianthedev/advent/internal/runner"
)

func run(args []string, input io.Reader, output io.Writer) error {
	if len(args) != 3 {
		return fmt.Errorf("usage: aoc-go YEAR DAY PART (1 or 2)")
	}
	values := make([]int, 3)
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
	if values[2] != 1 && values[2] != 2 {
		return fmt.Errorf("part must be 1 or 2")
	}
	data, err := io.ReadAll(input)
	if err != nil {
		return err
	}
	part := day.Part1
	if values[2] == 2 {
		part = day.Part2
	}
	_, err = fmt.Fprintln(output, part(string(data)))
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
