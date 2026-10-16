package d01

import (
	"github.com/alecthomas/participle/v2"
	"github.com/yurianthedev/advent/internal/runner"
)

type Expense struct {
	Value int `parser:"@Int"`
}

var parser = participle.MustBuild[Expense]()

func Process(input *runner.Input) ([]int, error) {
	records, err := runner.Collect(runner.ParseLines(input.Lines(), parser))
	if err != nil {
		return nil, err
	}
	numbers := make([]int, len(records))
	for i, record := range records {
		numbers[i] = record.Value
	}
	return numbers, nil
}

func Part1(data []int) (int, error) {
	var result int
	for i := range data {
		for j := 1; j < len(data); j++ {
			if i != j && data[i]+data[j] == 2020 {
				result = data[i] * data[j]
			}
		}
	}
	return result, nil
}

func Part2(data []int) (int, error) {
	var result int
	for i := range data {
		for j := 1; j < len(data); j++ {
			for m := 2; m < len(data); m++ {
				if i != j && i != m && j != m && data[i]+data[j]+data[m] == 2020 {
					result = data[i] * data[j] * data[m]
				}
			}
		}
	}
	return result, nil
}

var Examples = []runner.Example[int, int]{{
	Input: "1721\n979\n366\n299\n675\n1456\n",
	Part1: runner.Expect(514579), Part2: runner.Expect(241861950),
}}
var Solution = runner.New(Process, Part1, Part2, Examples...)
