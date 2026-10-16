# Writing a solution

Every language follows one contract:

1. Receive a byte-backed input with lazy views.
2. Run one shared `process` step.
3. Pass its result to the selected part.

The prepared result can remain a lazy stream or be an explicitly collected grid, vector, graph, etc. Each part gets a fresh byte source. Examples and real inputs use the same path. Raw parsing libraries remain directly available—there is no repository-specific parser DSL.

## Input views

Choose a view in `process`; nothing is decoded or collected before that choice.

| View | Meaning |
| --- | --- |
| `bytes` | Unmodified bytes, read in bounded chunks. No UTF-8 requirement. |
| `chars` | Incrementally decoded UTF-8 characters; malformed UTF-8 fails. |
| `lines` | Strings collected one line at a time, without a record-size limit. |
| `lineChars` / `line_chars` | A stream of character streams: even an individual line is read lazily. |
| `text` | Explicitly collect the complete UTF-8 text. |

Text/character views preserve original line endings. Line views remove LF, CRLF, or CR terminators while retaining spaces and blank lines; a final terminator does not invent another empty line. Scala uses native UTF-16 `Char` units, Rust/Go/Haskell Unicode scalar characters, and Elixir codepoint integers.

Consume each nested line **before advancing its outer stream**. Advancing discards unread characters in that line; retaining old borrowed line streams for later use is an error. Collect explicitly before storing or parallelizing them. Haskell uses persistent lazy lists rather than mutable borrowed cursors: retaining old prefixes retains their memory. Elixir's borrowed nested views must be consumed in the enumerating process. Treat the byte source and its alternative views as single-pass; only Haskell's persistent input is freely shareable.

Scala's `Input` supplies native Kyo streams, Go's `*runner.Input` supplies `runner.Stream` callbacks, Rust's `aoc::Input` supplies fallible iterators, Elixir's `Advent.Input` supplies enumerables, and Haskell's `AOC.Input` supplies lazy bytestrings/lists.

| Language | Parsing library | Processing API |
| --- | --- | --- |
| Scala | Native `kyo-parse` 1.0.0-RC7 | `Solution` with inner types |
| Rust | winnow 0.7 | `aoc::Solution` with associated types |
| Go | Participle v2 | `runner.New(Process, Part1, Part2, examples...)` |
| Elixir | NimbleParsec | `process/1` in an `Advent.Day` module |
| Haskell | Megaparsec | `solution year day process first second examples` |

The checked-in solutions use these APIs. Their existing algorithms and incomplete parts are retained. `wip/` remains an archive, outside active builds.

## Scala

A day is an executable Kyo application. Run its object from the IDE, or select its fully qualified main class with Scala CLI. No arguments are required.

```scala
package aoc.solutions.y2026

import aoc.*
import kyo.*

object Day01 extends Solution {
  type Prepared = Vector[Int]
  type Answer1 = Int // Answer2 defaults to Answer1

  def process(input: Input): Prepared < Puzzle.Effects =
    input.lines.map(line => Parse.runOrAbort(line)(Parse.entireInput(Parse.int)))
      .fold(Vector.empty[Int])(_ :+ _)

  override val part1 = solve((values: Prepared) => values.sum)

  override val examples = Seq(Example1("2\n3\n", 5))
}
```

This is a small API illustration, not a puzzle solution. `Parse.runOrAbort`, `Parse.int`, `Parse.inOrder`, `Parse.spaced`, `Parse.regex`, etc. are **Kyo's own APIs**. In RC7, use `Parse.entireInput(parser)` to reject trailing text. Scala 3.9.0, Kyo core/parse 1.0.0-RC7, and the native Kyo test API/runner are pinned together; MUnit is not used. The published RC7 jars require JDK 25 or newer.

`aoc.Parsing.nonNegativeLong` supplies one missing primitive: ASCII decimal digits bounded by `Long.MaxValue`, without accepting signs or consuming a following separator. It is exported by `Solution`, so a day can use `nonNegativeLong` directly. Overflow is a typed parse failure, not a numeric exception. Compose it with ordinary Kyo combinators; the helper is not a new parsing DSL.

`process` and parts can return computations using `Puzzle.Effects` (`Async`, `Abort[Throwable]`, and `Scope`) or a narrower effect such as `Any`. Pure values lift automatically. Kyo effects and failures are handled by the common application boundary.

### Available parts and typed examples

A part is unavailable until registered with `solve`. Omit it rather than register a body containing `???`. IDE execution skips absent parts. Explicit CLI selection of an unavailable part fails clearly; real errors in implemented parts are never interpreted as “unfinished.”

Examples form an ADT:

```scala
Example1(input, expected1)
Example2(input, expected2)
ExampleBoth(input, expected1, expected2)
ExampleInput(input)
```

`examples` has type `Seq[Example[Answer1, Answer2]]`. Expectations are native values, not their string representations: use `3L` for a `Long`, not `"3"`. Incorrect expectation types fail compilation. `ExampleInput` is useful before an expected answer is known; it can be run but supplies no assertion to `aoc test`.

When the two answers really differ, extend `Solution.WithAnswers` and declare both inner types. This uses the same implementation and runner. Scala cannot override a concrete type alias with a different type, so the opt-out is explicit.

### IDE behavior

Running a day object:

- Runs examples relevant to each available part and checks supplied expectations.
- Also runs `data/yYY/dDD/input.txt` when present.
- Skips absent examples, absent input, and unavailable parts.
- Does nothing when nothing is available; never fetches input automatically.

An example declaring only a part-2 expectation is not automatically used for part 1. An input-only example applies to every available part. File lookup honors `AOC_ROOT`, otherwise finds `aoc.toml` above the working directory. Open the repository/language project in the IDE so that lookup succeeds.

```sh
cd scala
scala-cli run . --server=false --main-class aoc.solutions.y2025.Day01
```

### Streaming and benchmarking

Keep `Prepared` as a Kyo stream to avoid collecting input. For example:

```scala
type Prepared = Stream[Int, Sync & Abort[ParseError]]

def process(input: Input): Prepared < Any =
  input.lines.map(line => Parse.runOrAbort(line)(Parse.entireInput(Parse.int)))

override val part1 = solve((values: Prepared) => values.fold(0)(_ + _))
```

**No replay override is needed for Kyo streams or ordinary strict immutable results.** Part registration automatically captures the prepared type: solve-only benchmarks materialize parsed streams once, then replay those records without reparsing. Strict immutable values, such as 2025 Day 1's `Vector`, are reused. Normal runs remain streaming and never create a snapshot.

Nested Kyo streams are snapshotted recursively while each inner stream is active. `input.lineChars` has type `Stream[Stream[Char, Sync], Sync]`, so benchmarking that prepared shape also needs no override. RC7 supports the required nested stream tags without a repository-specific workaround.

The optional `snapshot(input: Prepared): (() => Prepared) < Puzzle.Effects` hook remains available for customization. Override it for mutable state, streams hidden inside arbitrary containers, or other unusual prepared shapes; return a factory that makes fresh input for each solve-only iteration. For example, an array-based solution can return `() => input.clone()`. The default rejects known mutable and unsupported lazy shapes rather than silently benchmarking reused state. `aoc bench --wall` is also available.

`Day[A, B]`, `StreamingDay[A, B]`, and string-based `Sample` declarations remain compatibility APIs. They are not additional recommended solution styles. Their old raw-text/raw-byte access is retained for existing code.

## Rust

A day exports `Solver` implementing `aoc::Solution` and a typed `EXAMPLES` slice. The generated year registry selects it with `1 => d01: Solver`.

```rust
pub struct Solver;
impl aoc::Solution for Solver {
    type Prepared<'a> = Vec<i64>;
    type Answer = i64;
    const PARTS: &'static [u8] = &[1];

    fn process(input: aoc::Input<'_>) -> Result<Vec<i64>, String> {
        aoc::records(input.lines(), |line| {
            aoc::parse(line, aoc::winnow::ascii::dec_int)
        }).collect()
    }
    fn part1(values: Vec<i64>) -> Result<i64, String> {
        Ok(values.into_iter().sum())
    }
    fn snapshot(values: Vec<i64>) -> Result<Box<dyn Fn() -> Vec<i64>>, String> {
        Ok(Box::new(move || values.clone()))
    }
}
pub const EXAMPLES: &[aoc::Example<i64>] = &[
    aoc::example!("sum", "2\n3\n", part1 = 5),
];
```

`aoc::winnow` re-exports the native library. `aoc::parse` only handles whole-record parsing/error conversion; `aoc::records` lazily attaches line numbers and propagates IO errors. Compose winnow parsers directly. `Prepared<'a>` can instead be `aoc::Records<'a, Record>` to retain streaming.

`Answer` covers both parts. For different result types, implement `aoc::WithAnswers` with `Part1` and `Part2`; the runner is unchanged. `Example<A, B = A>` ties expectations to those result types at registration.

`PARTS` explicitly lists implemented parts. Its default is empty. Rust cannot inspect whether a method contains `todo!`, and the runner does not catch arbitrary failures to guess availability.

Unlike immutable Scala values, Rust parts own their prepared input. Solve-only benchmarks therefore need an explicit replay/copy factory. Streaming factories collect parsed records once, then create a new iterator over a copy per iteration. Use `--wall` when no replay is defined.

For an IDE/native run, run the year binary with no arguments, or select a day:

```sh
cargo run --manifest-path rust/Cargo.toml -p aoc-y2021 -- auto 7
```

It runs available examples and existing central input. Legacy module registration and `Sample`/`sample!` remain supported. Compile-time `aoc::input!` is still opt-in and is not used by the stream-first runner.

## Go

A day declares its Participle grammar using the native library, then registers typed functions:

```go
var parser = participle.MustBuild[Expense]()

func Process(input *runner.Input) ([]*Expense, error) {
    return runner.Collect(runner.ParseLines(input.Lines(), parser))
}
```

`runner.ParseLines` is just a lazy Participle bridge with line context. `runner.Stream[T]` is a single-pass callback iterator compatible with Go 1.19. Return `false` from its callback to stop early. Line reading has no Scanner-style 64 KiB limit.

`runner.New(Process, Part1, Part2, examples...)` infers prepared and answer types. A nil part is unavailable. Expectations are `runner.Example[A, B]`, with `runner.Expect(value)` supplying optional typed values. The generated registry discovers the package's exported `Solution` value; legacy packages with only string-based part functions still use `runner.Register`.

Native example tests call `Solution.CheckExamples(1)` / `(2)` in `TestExamples/part1` and `/part2`. The adapter also accepts legacy `TestSamples` names.

Go day packages are libraries, not forced into separate `main` packages. Run the native driver in the IDE with `auto YEAR DAY`, or no arguments for all registered days:

```sh
cd go
go run ./cmd/aoc auto 2020 1
```

Native benchmarks recursively materialize `runner.Stream` values, including nested character lines, and replay them. Ordinary value data, slices, maps, and exported record fields are copied outside solve-only timing. Opaque runtime objects, unsupported functions, and cyclic/deep structures are rejected; use `--wall` for those cases. The copier is for value-shaped puzzle data, not arbitrary object graphs with identity-sensitive aliases.

## Elixir

Keep `use Advent.Day, year: ..., day: ...`, add `process/1`, and use native NimbleParsec declarations:

```elixir
import NimbleParsec
defparsec :number, integer(min: 1) |> eos()

def process(input), do: Advent.Parse.lines(Advent.Input.lines(input), &number/1) |> Stream.map(&hd/1)
def part1(numbers), do: Enum.sum(numbers)
example "2\n3\n", part1: 5
```

`Advent.Parse.lines` is a lazy bridge to an ordinary NimbleParsec function; it rejects leftover input and adds line context. Expected answers are already native Elixir values. `example "input"` is allowed without expectations; `sample` is a compatibility spelling.

Omit unimplemented part functions. The runner discovers exported `part1/1` and `part2/1`; it does not treat exceptions as missing implementations. Every day gets `run/0` for native/IDE use:

```sh
cd elixir
mix run -e 'Advent.Y20.D02.run()'
```

The native Mix run task streams input too. When its stdin must serve both parts, it uses a private temporary file for replay. Native benchmarks recursively materialize stream enumerables, including nested character lines, and recreate them per solve-only iteration; ordinary immutable prepared values are reused. An optional module `snapshot/1` can return a custom replay factory.

## Haskell

Use a byte-backed `AOC.Input.Input`, native Megaparsec combinators, and the typed `solution` constructor:

```haskell
process = records (L.decimal :: Parser Int) . Input.lines
puzzle = solution 2026 1 process (Just sum) (Nothing :: Maybe ([Int] -> Int)) examples
main = runAvailable puzzle
examples = [Example1 "2\n3\n" 5]
```

`Parser`, `record`, and `records` live in `AOC.Parsing`; they only choose the error type, enforce end-of-record, and attach line context. The grammar is Megaparsec's. `records` is lazy. Collect/build a structure in `process` when the algorithm needs it.

`Example1`, `Example2`, `ExampleBoth`, and `ExampleInput` carry typed answers. Haskell infers the answer types from functions and fixtures. `Nothing` means an unavailable part; registered functions that throw are failures, not missing work.

```sh
cd haskell
runghc -isrc src/Year2021/Day06.hs
```

Solve-only benchmarks use `NFData` to fully evaluate prepared input before timing the solver. Immutable data can then be reused safely. Custom prepared records need an `NFData` instance; existing records use `Generic` derivation. The old text-parser `day` constructor and `Sample` values remain compatibility APIs.
