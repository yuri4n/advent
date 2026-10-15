package main

import (
	"fmt"
	"os"
	"strconv"

	"github.com/yurianthedev/advent/internal/scaffold"
)

func main() {
	if len(os.Args) != 3 {
		fmt.Fprintln(os.Stderr, "usage: go run ./cmd/scaffold YEAR DAY (from go/)")
		os.Exit(1)
	}
	year, yearErr := strconv.Atoi(os.Args[1])
	day, dayErr := strconv.Atoi(os.Args[2])
	if yearErr != nil || dayErr != nil {
		fmt.Fprintln(os.Stderr, "year and day must be integers")
		os.Exit(1)
	}
	dir, err := scaffold.New(".", year, day)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	fmt.Println("Created", dir)
}
