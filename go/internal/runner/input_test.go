package runner

import (
	"io"
	"strings"
	"testing"
)

type countedReader struct {
	io.Reader
	read int
}

func (r *countedReader) Read(p []byte) (int, error) {
	n, e := r.Reader.Read(p)
	r.read += n
	return n, e
}

func TestNestedLineBenchmarkReplaysConsumedCharacters(t *testing.T) {
	solve := func(rows Stream[Stream[rune]]) (int, error) {
		total := 0
		var failure error
		rows(func(row Stream[rune], err error) bool {
			if err != nil {
				failure = err
				return false
			}
			chars, err := Collect(row)
			failure = err
			total += len(chars)
			return err == nil
		})
		return total, failure
	}
	day := New(func(input *Input) (Stream[Stream[rune]], error) { return input.LineChars(), nil }, solve, solve)
	report, err := day.Benchmark(1, "ab\n雪\n", 2, 1)
	if err != nil || report.Answer != "3" {
		t.Fatalf("nested replay: %+v %v", report, err)
	}
}

func TestByteInputViews(t *testing.T) {
	text := "é雪🙂\r\n\nlast"
	chars, err := Collect(NewInput(strings.NewReader(text)).Chars())
	if err != nil || string(chars) != text {
		t.Fatalf("characters: %q %v", string(chars), err)
	}
	lines, err := Collect(NewInput(strings.NewReader(text)).Lines())
	if err != nil || strings.Join(lines, "|") != "é雪🙂||last" {
		t.Fatalf("lines: %q %v", lines, err)
	}
	source := &countedReader{Reader: strings.NewReader(strings.Repeat("a", 100000) + "\nnext")}
	NewInput(source).LineChars()(func(line Stream[rune], err error) bool {
		if err != nil {
			t.Fatal(err)
		}
		n := 0
		line(func(c rune, err error) bool {
			if err != nil {
				t.Fatal(err)
			}
			n++
			return n < 2
		})
		return false
	})
	if source.read > 4096 {
		t.Fatalf("buffered a complete line: %d bytes", source.read)
	}
	var prefixes []string
	NewInput(strings.NewReader("abcdef\r\nxyz\n")).LineChars()(func(line Stream[rune], err error) bool {
		if err != nil {
			t.Fatal(err)
		}
		var prefix []rune
		line(func(c rune, err error) bool {
			if err != nil {
				t.Fatal(err)
			}
			prefix = append(prefix, c)
			return len(prefix) < 2
		})
		prefixes = append(prefixes, string(prefix))
		return true
	})
	if strings.Join(prefixes, "|") != "ab|xy" {
		t.Fatalf("partial nested lines: %q", prefixes)
	}
	_, err = Collect(NewInput(strings.NewReader("\xff")).Chars())
	if err == nil {
		t.Fatal("accepted invalid UTF-8")
	}
}
