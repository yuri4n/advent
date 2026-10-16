package runner

import (
	"bufio"
	"fmt"
	"io"
	"strings"
	"unicode/utf8"
)

// Input borrows a byte source. Select one view; the runner owns the source.
type Input struct {
	source  io.Reader
	claimed bool
}

func NewInput(source io.Reader) *Input { return &Input{source: source} }
func (in *Input) reader() (*bufio.Reader, error) {
	if in.claimed {
		return nil, fmt.Errorf("input already consumed; use a fresh input")
	}
	in.claimed = true
	return bufio.NewReader(in.source), nil
}
func (in *Input) Bytes() Stream[byte] {
	return func(yield func(byte, error) bool) {
		r, e := in.reader()
		if e != nil {
			yield(0, e)
			return
		}
		for {
			b, e := r.ReadByte()
			if e == io.EOF {
				return
			}
			if !yield(b, e) || e != nil {
				return
			}
		}
	}
}
func readRune(r *bufio.Reader) (rune, error) {
	c, size, e := r.ReadRune()
	if e == nil && c == utf8.RuneError && size == 1 {
		return 0, fmt.Errorf("invalid UTF-8 input")
	}
	return c, e
}
func (in *Input) Chars() Stream[rune] {
	return func(yield func(rune, error) bool) {
		r, e := in.reader()
		if e != nil {
			yield(0, e)
			return
		}
		for {
			c, e := readRune(r)
			if e == io.EOF {
				return
			}
			if !yield(c, e) || e != nil {
				return
			}
		}
	}
}

// LineChars yields borrowed lines without collecting their contents. Advancing
// the outer stream discards the unread remainder and expires the previous line.
func (in *Input) LineChars() Stream[Stream[rune]] {
	return func(yield func(Stream[rune], error) bool) {
		r, e := in.reader()
		if e != nil {
			yield(nil, e)
			return
		}
		generation := 0
		ended := true
		next := func() (rune, error) {
			if ended {
				return 0, io.EOF
			}
			c, e := readRune(r)
			if e != nil {
				ended = true
				return 0, e
			}
			if c == '\r' || c == '\n' {
				if c == '\r' {
					b, e := r.Peek(1)
					if e != nil && e != io.EOF {
						return 0, e
					}
					if len(b) > 0 && b[0] == '\n' {
						_, _ = r.Discard(1)
					}
				}
				ended = true
				return 0, io.EOF
			}
			return c, nil
		}
		defer func() { generation++ }()
		for {
			for !ended {
				_, e = next()
				if e != nil && e != io.EOF {
					yield(nil, e)
					return
				}
			}
			generation++
			if _, e = r.Peek(1); e != nil {
				if e != io.EOF {
					yield(nil, e)
				}
				return
			}
			ended = false
			id := generation
			used := false
			line := Stream[rune](func(emit func(rune, error) bool) {
				if id != generation || used {
					emit(0, fmt.Errorf("line view expired or already consumed"))
					return
				}
				used = true
				for {
					c, e := next()
					if e == io.EOF {
						return
					}
					if !emit(c, e) || e != nil {
						return
					}
				}
			})
			if !yield(line, nil) {
				return
			}
		}
	}
}
func (in *Input) Lines() Stream[string] {
	return func(yield func(string, error) bool) {
		in.LineChars()(func(line Stream[rune], err error) bool {
			if err != nil {
				yield("", err)
				return false
			}
			var text strings.Builder
			line(func(c rune, e error) bool {
				if e != nil {
					err = e
					return false
				}
				_, _ = text.WriteRune(c)
				return true
			})
			return yield(text.String(), err) && err == nil
		})
	}
}

// Text is the explicit whole-input UTF-8 collection operation.
func (in *Input) Text() (string, error) {
	r, e := in.reader()
	if e != nil {
		return "", e
	}
	data, e := io.ReadAll(r)
	if e != nil {
		return "", e
	}
	if !utf8.Valid(data) {
		return "", fmt.Errorf("invalid UTF-8 input")
	}
	return string(data), nil
}
