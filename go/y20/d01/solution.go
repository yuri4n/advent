package d01

import (
	"strconv"
	"strings"
)

func parse(input string) []int {
	lines := strings.Fields(input)
	numbers := make([]int, len(lines))
	for i, line := range lines {
		value, err := strconv.Atoi(line)
		if err != nil {
			panic(err)
		}
		numbers[i] = value
	}
	return numbers
}

func Part1(input string) int {
	data := parse(input)
	var result int
	for i := range data {
		for j := 1; j < len(data); j++ {
			if i != j && data[i]+data[j] == 2020 {
				result = data[i] * data[j]
			}
		}
	}
	return result
}

func Part2(input string) int {
	data := parse(input)
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
	return result
}
