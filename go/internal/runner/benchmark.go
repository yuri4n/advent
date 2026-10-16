package runner

import (
	"fmt"
	"io"
	"reflect"
	"strings"
	"time"
)

type Benchmark struct {
	Answer  string
	Setup   int64
	Timings [][2]int64
}

func (report Benchmark) Print(out io.Writer) error {
	if _, err := fmt.Fprintf(out, "AOC_BENCH_V1\n%d\n", report.Setup); err != nil {
		return err
	}
	for _, pair := range report.Timings {
		if _, err := fmt.Fprintf(out, "%d %d\n", pair[0], pair[1]); err != nil {
			return err
		}
	}
	_, err := fmt.Fprintln(out, report.Answer)
	return err
}
func (day Day) Benchmark(part int, text string, iterations, warmup int) (Benchmark, error) {
	if part < 1 || part > 2 || day.parts[part-1] == nil {
		return Benchmark{}, fmt.Errorf("part %d is unavailable", part)
	}
	if day.bench == nil {
		return Benchmark{}, fmt.Errorf("legacy day: use aoc bench --wall")
	}
	return day.bench(part, text, iterations, warmup)
}

func benchmark[P any](text string, iterations, warmup int, process func(*Input) (P, error), solve func(P) (any, error)) (Benchmark, error) {
	report := Benchmark{}
	if iterations < 1 || iterations > 10000 || warmup < 0 || warmup > 1000 {
		return report, fmt.Errorf("invalid benchmark counts")
	}
	var expected *string
	check := func(value any) error {
		answer := fmt.Sprint(value)
		if expected != nil && *expected != answer {
			return fmt.Errorf("answer changed between benchmark runs or modes")
		}
		expected = &answer
		report.Answer = answer
		return nil
	}
	totals := make([]int64, 0, iterations)
	for i := 0; i < iterations+warmup; i++ {
		start := time.Now()
		prepared, err := process(NewInput(strings.NewReader(text)))
		if err != nil {
			return report, err
		}
		answer, err := solve(prepared)
		if err != nil {
			return report, err
		}
		answer = fmt.Sprint(answer)
		elapsed := time.Since(start).Nanoseconds()
		if err := check(answer); err != nil {
			return report, err
		}
		if i >= warmup {
			totals = append(totals, elapsed)
		}
	}
	start := time.Now()
	prepared, err := process(NewInput(strings.NewReader(text)))
	if err != nil {
		return report, err
	}
	replay, err := freeze(prepared)
	if err != nil {
		return report, err
	}
	report.Setup = time.Since(start).Nanoseconds()
	for i := 0; i < iterations+warmup; i++ {
		fresh, err := replay()
		if err != nil {
			return report, err
		}
		start := time.Now()
		answer, err := solve(fresh.(P))
		if err != nil {
			return report, err
		}
		answer = fmt.Sprint(answer)
		elapsed := time.Since(start).Nanoseconds()
		if err := check(answer); err != nil {
			return report, err
		}
		if i >= warmup {
			report.Timings = append(report.Timings, [2]int64{totals[i-warmup], elapsed})
		}
	}
	return report, nil
}

type replayable interface {
	replay() (func() (any, error), error)
}

func freeze(input any) (func() (any, error), error) {
	if stream, ok := input.(replayable); ok {
		return stream.replay()
	}
	// Validate the value shape before reporting benchmark setup as successful.
	if _, err := cloneValue(reflect.ValueOf(input), 0); err != nil {
		return nil, err
	}
	return func() (any, error) {
		if input == nil {
			return nil, nil
		}
		copy, err := cloneValue(reflect.ValueOf(input), 0)
		if err != nil {
			return nil, err
		}
		return copy.Interface(), nil
	}, nil
}
func (stream Stream[T]) replay() (func() (any, error), error) {
	var saved []func() (any, error)
	var failure error
	stream(func(value T, err error) bool {
		if err != nil {
			failure = err
			return false
		}
		replay, err := freeze(value)
		if err != nil {
			failure = err
			return false
		}
		saved = append(saved, replay)
		return true
	})
	if failure != nil {
		return nil, failure
	}
	return func() (any, error) {
		values := make([]T, len(saved))
		for i, replay := range saved {
			value, err := replay()
			if err != nil {
				return nil, err
			}
			if value != nil {
				values[i] = value.(T)
			}
		}
		used := false
		return Stream[T](func(yield func(T, error) bool) {
			if used {
				var zero T
				yield(zero, fmt.Errorf("input stream already consumed"))
				return
			}
			used = true
			for _, value := range values {
				if !yield(value, nil) {
					return
				}
			}
		}), nil
	}, nil
}

// Benchmark-only copying of ordinary value data. Opaque runtime objects and
// cyclic graphs are rejected rather than reused with potentially mutated state.
func cloneValue(value reflect.Value, depth int) (reflect.Value, error) {
	if !value.IsValid() {
		return value, nil
	}
	if depth > 100 {
		return reflect.Value{}, fmt.Errorf("cyclic/deep benchmark input: use --wall")
	}
	copy := reflect.New(value.Type()).Elem()
	switch value.Kind() {
	case reflect.Bool, reflect.String, reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64,
		reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64, reflect.Uintptr,
		reflect.Float32, reflect.Float64, reflect.Complex64, reflect.Complex128:
		return value, nil
	case reflect.Interface, reflect.Ptr:
		if value.IsNil() {
			return copy, nil
		}
		inner, err := cloneValue(value.Elem(), depth+1)
		if err != nil {
			return copy, err
		}
		if value.Kind() == reflect.Ptr {
			copy.Set(reflect.New(value.Type().Elem()))
			copy.Elem().Set(inner)
		} else {
			copy.Set(inner)
		}
	case reflect.Slice, reflect.Array:
		if value.Kind() == reflect.Slice {
			if value.IsNil() {
				return copy, nil
			}
			copy = reflect.MakeSlice(value.Type(), value.Len(), value.Len())
		}
		for i := 0; i < value.Len(); i++ {
			item, err := cloneValue(value.Index(i), depth+1)
			if err != nil {
				return copy, err
			}
			copy.Index(i).Set(item)
		}
	case reflect.Struct:
		for i := 0; i < value.NumField(); i++ {
			if !value.Type().Field(i).IsExported() {
				return copy, fmt.Errorf("opaque benchmark input %s: use --wall", value.Type())
			}
			item, err := cloneValue(value.Field(i), depth+1)
			if err != nil {
				return copy, err
			}
			copy.Field(i).Set(item)
		}
	case reflect.Map:
		if value.IsNil() {
			return copy, nil
		}
		copy = reflect.MakeMapWithSize(value.Type(), value.Len())
		iter := value.MapRange()
		for iter.Next() {
			item, err := cloneValue(iter.Value(), depth+1)
			if err != nil {
				return copy, err
			}
			copy.SetMapIndex(iter.Key(), item)
		}
	default:
		return copy, fmt.Errorf("unsupported benchmark input %s: use --wall", value.Type())
	}
	return copy, nil
}
