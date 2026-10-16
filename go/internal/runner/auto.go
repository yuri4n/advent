package runner

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
)

func Root() string {
	if root := os.Getenv("AOC_ROOT"); root != "" {
		return root
	}
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for {
		if info, err := os.Stat(filepath.Join(dir, "aoc.toml")); err == nil && !info.IsDir() {
			return dir
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return ""
		}
		dir = parent
	}
}

// Auto runs only registered parts. Missing fixtures/input are not failures.
func (day Day) Auto(key Key, root string, output io.Writer) error {
	for index, solve := range day.parts {
		if solve == nil {
			continue
		}
		part := index + 1
		for i, example := range day.examples {
			expected := example.Part1
			if part == 2 {
				expected = example.Part2
			}
			if expected == nil && (example.Part1 != nil || example.Part2 != nil) {
				continue
			}
			actual, err := day.Run(part, strings.NewReader(example.Input))
			if err != nil {
				return err
			}
			if expected != nil && fmt.Sprint(actual) != *expected {
				return fmt.Errorf("example %d part %d: answer mismatch", i+1, part)
			}
			if _, err := fmt.Fprintf(output, "%d/%d example %d part %d: %v\n", key.Year, key.Day, i+1, part, actual); err != nil {
				return err
			}
		}
		if root == "" {
			continue
		}
		path := filepath.Join(root, fmt.Sprintf("data/y%02d/d%02d/input.txt", key.Year%100, key.Day))
		file, err := os.Open(path)
		if os.IsNotExist(err) {
			continue
		}
		if err != nil {
			return err
		}
		actual, err := day.Run(part, file)
		closeErr := file.Close()
		if err != nil {
			return err
		}
		if closeErr != nil {
			return closeErr
		}
		if _, err := fmt.Fprintf(output, "%d/%d input part %d: %v\n", key.Year, key.Day, part, actual); err != nil {
			return err
		}
	}
	return nil
}
