package main

import (
	"bytes"
	"strings"
	"testing"
)

func TestRun(t *testing.T) {
	for _, test := range []struct {
		part, want string
	}{{"1", "514579\n"}, {"2", "241861950\n"}} {
		var output bytes.Buffer
		err := run([]string{"2020", "1", test.part}, strings.NewReader("1721\n979\n366\n299\n675\n1456\n"), &output)
		if err != nil || output.String() != test.want {
			t.Fatalf("part %s: output %q, error %v; want %q", test.part, output.String(), err, test.want)
		}
	}
}

func TestRunRejectsUnknownDayAndPart(t *testing.T) {
	for _, args := range [][]string{{"2020", "2", "1"}, {"2020", "1", "3"}, {"wrong", "1", "1"}} {
		if err := run(args, strings.NewReader(""), &bytes.Buffer{}); err == nil {
			t.Fatalf("accepted invalid arguments %v", args)
		}
	}
}
