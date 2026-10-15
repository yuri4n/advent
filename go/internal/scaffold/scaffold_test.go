package scaffold

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestNewCreatesDiscoverableStubsAndPreservesExistingDay(t *testing.T) {
	root := t.TempDir()
	dir, err := New(root, 2026, 1)
	if err != nil {
		t.Fatal(err)
	}
	source, err := os.ReadFile(filepath.Join(dir, "solution.go"))
	if err != nil || !strings.Contains(string(source), `panic("TODO: implement part 1")`) {
		t.Fatalf("missing empty stub: %q, %v", source, err)
	}
	registry, err := os.ReadFile(filepath.Join(root, "cmd", "aoc", "days_gen.go"))
	if err != nil || !strings.Contains(string(registry), "runner.Register(y26d01.Part1, y26d01.Part2)") {
		t.Fatalf("new day not registered: %q, %v", registry, err)
	}
	if _, err := New(root, 2026, 1); err == nil {
		t.Fatal("existing day was overwritten")
	}
	after, _ := os.ReadFile(filepath.Join(dir, "solution.go"))
	if string(after) != string(source) {
		t.Fatal("existing solution changed")
	}
}

func TestNewRollsBackWhenRegistryIsUserOwned(t *testing.T) {
	root := t.TempDir()
	path := filepath.Join(root, "cmd", "aoc")
	if err := os.MkdirAll(path, 0755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(path, "days_gen.go"), []byte("package main\n"), 0644); err != nil {
		t.Fatal(err)
	}
	if _, err := New(root, 2026, 1); err == nil {
		t.Fatal("overwrote user registry")
	}
	if _, err := os.Stat(filepath.Join(root, "y26", "d01")); !os.IsNotExist(err) {
		t.Fatal("failed generation left a partial day")
	}
}
