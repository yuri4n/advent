//! Stream-first Advent of Code harness with winnow parsing.
//!
//! A year registers normal Rust modules with [`main!`]. Each module exports
//! `part1(&str)`, `part2(&str)` (any `Display` answer), and `SAMPLES`.

use std::{
    fmt::Display,
    io::{self, BufRead},
    panic::AssertUnwindSafe,
    time::Instant,
};
pub use winnow;
mod input;
pub use input::{Chars, Input, LineChars};

pub type Lines<'a> = Box<dyn Iterator<Item = io::Result<String>> + 'a>;
pub type Records<'a, T> = Box<dyn Iterator<Item = Result<T, String>> + 'a>;

/// The usual case: one answer type, one shared processing function.
pub trait Solution {
    type Prepared<'a>;
    type Answer: Display;
    const PARTS: &'static [u8] = &[];
    fn process(input: Input<'_>) -> Result<Self::Prepared<'_>, String>;
    fn part1(_input: Self::Prepared<'_>) -> Result<Self::Answer, String> {
        Err("part 1 is unavailable".into())
    }
    fn part2(_input: Self::Prepared<'_>) -> Result<Self::Answer, String> {
        Err("part 2 is unavailable".into())
    }
    fn snapshot(
        _input: Self::Prepared<'_>,
    ) -> Result<Box<dyn Fn() -> Self::Prepared<'static>>, String> {
        Err("implement snapshot for solve-only benchmarking, or use aoc bench --wall".into())
    }
}

/// Opt out of the shared answer type without changing the processing API.
pub trait WithAnswers {
    type Prepared<'a>;
    type Part1: Display;
    type Part2: Display;
    const PARTS: &'static [u8] = &[];
    fn process(input: Input<'_>) -> Result<Self::Prepared<'_>, String>;
    fn part1(_input: Self::Prepared<'_>) -> Result<Self::Part1, String> {
        Err("part 1 is unavailable".into())
    }
    fn part2(_input: Self::Prepared<'_>) -> Result<Self::Part2, String> {
        Err("part 2 is unavailable".into())
    }
    fn snapshot(
        _input: Self::Prepared<'_>,
    ) -> Result<Box<dyn Fn() -> Self::Prepared<'static>>, String> {
        Err("implement snapshot for solve-only benchmarking, or use aoc bench --wall".into())
    }
}
impl<T: Solution> WithAnswers for T {
    type Prepared<'a> = T::Prepared<'a>;
    type Part1 = T::Answer;
    type Part2 = T::Answer;
    const PARTS: &'static [u8] = T::PARTS;
    fn process(input: Input<'_>) -> Result<Self::Prepared<'_>, String> {
        T::process(input)
    }
    fn part1(input: Self::Prepared<'_>) -> Result<Self::Part1, String> {
        T::part1(input)
    }
    fn part2(input: Self::Prepared<'_>) -> Result<Self::Part2, String> {
        T::part2(input)
    }
    fn snapshot(
        input: Self::Prepared<'_>,
    ) -> Result<Box<dyn Fn() -> Self::Prepared<'static>>, String> {
        T::snapshot(input)
    }
}

pub struct Benchmark {
    pub answer: String,
    pub setup_ns: u128,
    pub timings: Vec<(u128, u128)>,
}
impl Benchmark {
    pub fn print(&self) {
        println!("AOC_BENCH_V1\n{}", self.setup_ns);
        for (total, solve) in &self.timings {
            println!("{total} {solve}");
        }
        println!("{}", self.answer);
    }
}

fn bench<S: WithAnswers>(
    part: u8,
    text: &[u8],
    iterations: u32,
    warmup: u32,
) -> Result<Benchmark, String> {
    fn solve<S: WithAnswers>(part: u8, prepared: S::Prepared<'_>) -> Result<String, String> {
        let prepared = std::hint::black_box(prepared);
        match part {
            1 => S::part1(prepared).map(|value| value.to_string()),
            2 => S::part2(prepared).map(|value| value.to_string()),
            _ => Err("part must be 1 or 2".into()),
        }
    }
    let mut expected = None;
    let mut check = |answer: String| -> Result<(), String> {
        if expected
            .as_ref()
            .is_some_and(|previous| previous != &answer)
        {
            return Err("answer changed between benchmark runs or modes".into());
        }
        expected = Some(answer);
        Ok(())
    };
    let mut totals = Vec::new();
    for index in 0..iterations + warmup {
        let reader = io::Cursor::new(std::hint::black_box(text));
        let started = Instant::now();
        let prepared = S::process(Input::new(reader))?;
        let answer = solve::<S>(part, prepared)?;
        let elapsed = started.elapsed().as_nanos();
        check(answer)?;
        if index >= warmup {
            totals.push(elapsed);
        }
    }
    let reader = io::Cursor::new(text);
    let started = Instant::now();
    let replay = S::snapshot(S::process(Input::new(reader))?)?;
    let setup_ns = started.elapsed().as_nanos();
    let mut timings = Vec::new();
    for index in 0..iterations + warmup {
        let prepared = replay();
        let started = Instant::now();
        let answer = solve::<S>(part, prepared)?;
        let elapsed = started.elapsed().as_nanos();
        check(answer)?;
        if index >= warmup {
            timings.push((totals[(index - warmup) as usize], elapsed));
        }
    }
    Ok(Benchmark {
        answer: expected.unwrap(),
        setup_ns,
        timings,
    })
}

/// Parse a complete record, rejecting trailing junk.
pub fn parse<'a, T>(
    input: &'a str,
    mut parser: impl winnow::Parser<&'a str, T, winnow::error::ContextError>,
) -> Result<T, String> {
    parser.parse(input).map_err(|error| error.to_string())
}

/// Lazy record parsing; IO and parse errors keep their line number.
pub fn records<'a, T: 'a>(
    lines: Lines<'a>,
    mut parse: impl FnMut(&str) -> Result<T, String> + 'a,
) -> Records<'a, T> {
    Box::new(lines.enumerate().map(move |(index, line)| {
        line.map_err(|error| error.to_string())
            .and_then(|line| parse(&line))
            .map_err(|error| format!("line {}: {error}", index + 1))
    }))
}

#[derive(Clone, Copy, Debug)]
pub struct Example<A = &'static str, B = A> {
    pub name: &'static str,
    pub input: &'static str,
    pub part1: Option<A>,
    pub part2: Option<B>,
}

pub type Sample = Example;

pub struct StoredExample {
    pub name: &'static str,
    pub input: &'static str,
    pub part1: Option<String>,
    pub part2: Option<String>,
}
fn store_examples<A: Display, B: Display>(examples: &[Example<A, B>]) -> Vec<StoredExample> {
    examples
        .iter()
        .map(|example| StoredExample {
            name: example.name,
            input: example.input,
            part1: example.part1.as_ref().map(ToString::to_string),
            part2: example.part2.as_ref().map(ToString::to_string),
        })
        .collect()
}

type StreamSolver = fn(u8, &mut dyn BufRead) -> Result<String, String>;
type NativeBenchmark = fn(u8, &[u8], u32, u32) -> Result<Benchmark, String>;
enum Solver {
    Text(fn(&str) -> String, fn(&str) -> String),
    Stream(StreamSolver),
}

pub struct Day {
    pub number: u8,
    solver: Solver,
    pub samples: Vec<StoredExample>,
    pub available_parts: &'static [u8],
    bench: Option<NativeBenchmark>,
}

impl Day {
    pub fn text(
        number: u8,
        part1: fn(&str) -> String,
        part2: fn(&str) -> String,
        examples: &'static [Example],
    ) -> Self {
        Self {
            number,
            solver: Solver::Text(part1, part2),
            samples: store_examples(examples),
            available_parts: &[1, 2],
            bench: None,
        }
    }
    pub fn processed<S: WithAnswers>(number: u8, examples: &[Example<S::Part1, S::Part2>]) -> Self {
        Self {
            number,
            samples: store_examples(examples),
            available_parts: S::PARTS,
            bench: Some(bench::<S>),
            solver: Solver::Stream(|part, reader| {
                let prepared = S::process(Input::new(reader))?;
                match part {
                    1 => S::part1(prepared).map(|answer| answer.to_string()),
                    2 => S::part2(prepared).map(|answer| answer.to_string()),
                    _ => Err("part must be 1 or 2".into()),
                }
            }),
        }
    }
    pub fn solve(&self, part: u8, input: &mut dyn BufRead) -> Result<String, String> {
        if !self.available_parts.contains(&part) {
            return Err(format!("part {part} is unavailable"));
        }
        std::panic::catch_unwind(AssertUnwindSafe(|| match self.solver {
            Solver::Stream(solve) => solve(part, input),
            Solver::Text(first, second) => {
                let mut text = String::new();
                input
                    .read_to_string(&mut text)
                    .map_err(|error| error.to_string())?;
                Ok(if part == 1 {
                    first(&text)
                } else {
                    second(&text)
                })
            }
        }))
        .unwrap_or_else(|payload| {
            let message = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("solution panicked");
            Err(format!("day {} part {part}: {message}", self.number))
        })
    }
    pub fn benchmark(
        &self,
        part: u8,
        text: &str,
        iterations: u32,
        warmup: u32,
    ) -> Result<Benchmark, String> {
        self.benchmark_bytes(part, text.as_bytes(), iterations, warmup)
    }
    pub fn benchmark_bytes(
        &self,
        part: u8,
        bytes: &[u8],
        iterations: u32,
        warmup: u32,
    ) -> Result<Benchmark, String> {
        if !(1..=10000).contains(&iterations) || warmup > 1000 {
            return Err("invalid benchmark counts".into());
        }
        if !self.available_parts.contains(&part) {
            return Err(format!("part {part} is unavailable"));
        }
        self.bench.ok_or("legacy day: use aoc bench --wall")?(part, bytes, iterations, warmup)
    }

    pub fn example(&self, index: usize) -> Result<&'static str, String> {
        index
            .checked_sub(1)
            .and_then(|index| self.samples.get(index))
            .map(|example| example.input)
            .ok_or_else(|| {
                format!(
                    "Example {index} does not exist; this day declares {} example(s)",
                    self.samples.len()
                )
            })
    }
}

/// Define an inline sample with one or both expected answers, written as strings.
/// Omit an expected answer while its part is unfinished.
#[macro_export]
macro_rules! example {
    ($name:expr, $input:expr $(,)?) => {
        $crate::Example {
            name: $name,
            input: $input,
            part1: None,
            part2: None,
        }
    };
    ($name:expr, $input:expr, part1 = $first:expr, part2 = $second:expr $(,)?) => {
        $crate::Example {
            name: $name,
            input: $input,
            part1: Some($first),
            part2: Some($second),
        }
    };
    ($name:expr, $input:expr, part1 = $first:expr $(,)?) => {
        $crate::Example {
            name: $name,
            input: $input,
            part1: Some($first),
            part2: None,
        }
    };
    ($name:expr, $input:expr, part2 = $second:expr $(,)?) => {
        $crate::Example {
            name: $name,
            input: $input,
            part1: None,
            part2: Some($second),
        }
    };
}

/// Compatibility spelling for existing solutions.
#[macro_export]
macro_rules! sample {
    ($($tokens:tt)*) => { $crate::example!($($tokens)*) };
}

/// Explicit opt-in compile-time input, relative to the calling crate manifest.
///
/// From `rust/years/y2021`, for example:
/// `aoc::input!("../../../data/y21/d07/input.txt")`.
/// Ordinary harness builds read stdin and never need private puzzle files.
#[macro_export]
macro_rules! input {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
    };
}

/// Register days using ordinary modules; no linker inventory or build script.
///
/// `aoc::main! { 7 => d07, 8 => d08 }`
#[macro_export]
macro_rules! main {
    ($($number:literal => $module:ident $(: $solution:ident)?),+ $(,)?) => {
        $(mod $module;)+

        fn main() {
            let days = &[$($crate::main!(@day $number, $module $(, $solution)?)),+];
            let arguments: Vec<String> = std::env::args().skip(1).collect();
            if arguments.is_empty() || arguments.first().is_some_and(|arg| arg == "auto") {
                let year = env!("CARGO_PKG_NAME").strip_prefix("aoc-y").and_then(|year| year.parse::<u16>().ok());
                let result = year.ok_or_else(|| "year crate must be named aoc-yYYYY".to_owned()).and_then(|year| {
                    let selected = arguments.get(1).map(|day| day.parse::<u8>().map_err(|_| "invalid day".to_owned())).transpose()?;
                    $crate::auto(days, year, selected, std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
                });
                if let Err(error) = result { eprintln!("{error}"); std::process::exit(1); }
            } else {
                std::process::exit($crate::dispatch(days, arguments.into_iter()));
            }
        }

        #[cfg(test)]
        mod inline_samples {
            $(#[test]
            fn $module() {
                use super::$module;
                let day = $crate::main!(@day $number, $module $(, $solution)?);
                // Native tests validate the expectations that have been supplied.
                // `aoc test` additionally requires coverage for the requested parts.
                $crate::check_samples(&day, None, false).unwrap();
            })+
        }
    };
    (@day $number:literal, $module:ident, $solution:ident) => {
        $crate::Day::processed::<$module::$solution>($number, &$module::EXAMPLES)
    };
    (@day $number:literal, $module:ident) => {
        $crate::Day::text($number, |input| $module::part1(input).to_string(),
            |input| $module::part2(input).to_string(), $module::SAMPLES)
    };
}

/// IDE/native execution: run available examples and input, without downloads.
pub fn auto(
    days: &[Day],
    year: u16,
    selected: Option<u8>,
    project: &std::path::Path,
) -> Result<(), String> {
    use std::{fs::File, path::PathBuf};
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let root = std::env::var_os("AOC_ROOT").map(PathBuf::from).or_else(|| {
        cwd.ancestors()
            .chain(project.ancestors())
            .find(|path| path.join("aoc.toml").is_file())
            .map(PathBuf::from)
    });
    if selected.is_some_and(|number| !days.iter().any(|day| day.number == number)) {
        return Err("day is not registered".into());
    }
    for day in days
        .iter()
        .filter(|day| selected.is_none_or(|number| number == day.number))
    {
        for &part in day.available_parts {
            for (index, example) in day.samples.iter().enumerate() {
                let expected = if part == 1 {
                    &example.part1
                } else {
                    &example.part2
                };
                if expected.is_some() || (example.part1.is_none() && example.part2.is_none()) {
                    let value = solve(day, part, example.input)?;
                    if expected.as_ref().is_some_and(|expected| expected != &value) {
                        return Err(format!(
                            "example {} part {part}: answer mismatch",
                            index + 1
                        ));
                    }
                    println!(
                        "{year}/{} example {} part {part}: {value}",
                        day.number,
                        index + 1
                    );
                }
            }
            if let Some(root) = &root {
                let path = root.join(format!(
                    "data/y{:02}/d{:02}/input.txt",
                    year % 100,
                    day.number
                ));
                match File::open(path) {
                    Ok(file) => println!(
                        "{year}/{} input part {part}: {}",
                        day.number,
                        day.solve(part, &mut io::BufReader::new(file))?
                    ),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.to_string()),
                }
            }
        }
    }
    Ok(())
}

fn solve(day: &Day, part: u8, input: &str) -> Result<String, String> {
    day.solve(part, &mut io::Cursor::new(input))
}

/// Validate provided expectations. Strict mode also requires every selected part
/// to have at least one expectation; missing coverage never counts as a pass.
pub fn check_samples(day: &Day, part: Option<u8>, strict: bool) -> Result<usize, String> {
    if part.is_some_and(|part| part != 1 && part != 2) {
        return Err("part must be 1 or 2".into());
    }
    let mut checked = [0usize; 2];
    let mut failures = Vec::new();
    for sample in &day.samples {
        for current in 1..=2 {
            if part.is_some_and(|part| part != current) {
                continue;
            }
            let expected = if current == 1 {
                sample.part1.as_deref()
            } else {
                sample.part2.as_deref()
            };
            if let Some(expected) = expected {
                checked[usize::from(current - 1)] += 1;
                match solve(day, current, sample.input) {
                    Ok(answer) if answer == expected => {}
                    Ok(answer) => failures.push(format!(
                        "{} part {current}: expected {expected:?}, got {answer:?}",
                        sample.name
                    )),
                    Err(error) => failures.push(format!("{}: {error}", sample.name)),
                }
            }
        }
    }
    if strict {
        for current in 1..=2 {
            if part.is_none_or(|part| part == current) && checked[usize::from(current - 1)] == 0 {
                failures.push(format!(
                    "part {current}: incomplete (no inline sample expectation)"
                ));
            }
        }
    } else if checked.iter().sum::<usize>() == 0 {
        failures.push("incomplete (no inline sample expectations)".into());
    }
    if failures.is_empty() {
        Ok(checked.iter().sum())
    } else {
        Err(failures.join("\n"))
    }
}

/// Native year-binary interface used by the repository language adapter.
pub fn dispatch(days: &[Day], arguments: impl Iterator<Item = String>) -> i32 {
    let args: Vec<String> = arguments.collect();
    match dispatch_result(days, &args) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

fn dispatch_result(days: &[Day], args: &[String]) -> Result<(), String> {
    if args.len() == 1 && args[0] == "list" {
        for day in days {
            println!("{}", day.number);
        }
        return Ok(());
    }
    let valid = match args.first().map(String::as_str) {
        Some("bench") => args.len() == 5,
        Some("expect") => args.len() == 4,
        Some("run" | "example") => args.len() == 3,
        Some("test") => args.len() == 2 || args.len() == 3,
        _ => false,
    };
    if !valid {
        return Err("usage: <year-binary> run DAY PART | test DAY [1|2|both] | list".into());
    }
    let number = args[1].parse::<u8>().map_err(|_| "invalid day")?;
    let day = days
        .iter()
        .find(|day| day.number == number)
        .ok_or_else(|| format!("day {number} is not registered"))?;
    if args[0] == "bench" {
        use std::io::Read;
        let part = args
            .get(2)
            .ok_or("missing part")?
            .parse()
            .map_err(|_| "invalid part")?;
        let iterations = args
            .get(3)
            .ok_or("missing iterations")?
            .parse()
            .map_err(|_| "invalid iterations")?;
        let warmup = args
            .get(4)
            .ok_or("missing warmup")?
            .parse()
            .map_err(|_| "invalid warmup")?;
        let mut bytes = Vec::new();
        io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        day.benchmark_bytes(part, &bytes, iterations, warmup)?
            .print();
        return Ok(());
    }
    if args[0] == "expect" {
        let part = args[2].parse::<u8>().map_err(|_| "invalid part")?;
        let index = args[3]
            .parse::<usize>()
            .map_err(|_| "invalid example index")?;
        day.example(index)?;
        let example = &day.samples[index - 1];
        let expected = match part {
            1 => &example.part1,
            2 => &example.part2,
            _ => return Err("part must be 1 or 2".into()),
        };
        match expected {
            Some(value) => println!("AOC_EXPECT_V1\n{value}"),
            None => println!("AOC_NO_EXPECT_V1"),
        }
        return Ok(());
    }
    if args[0] == "example" {
        let index = args
            .get(2)
            .ok_or("example requires an index")?
            .parse()
            .map_err(|_| "invalid example index")?;
        print!("{}", day.example(index)?);
        return Ok(());
    }
    let part = match args.get(2).map(String::as_str).unwrap_or("both") {
        "1" => Some(1),
        "2" => Some(2),
        "both" => None,
        _ => return Err("part must be 1, 2 or both".into()),
    };
    match args[0].as_str() {
        "run" => {
            let part = part.ok_or("run requires part 1 or 2")?;
            println!("{}", day.solve(part, &mut io::stdin().lock())?);
        }
        "test" => {
            let count = check_samples(day, part, true)?;
            println!("day {number}: {count} inline example expectation(s) passed");
        }
        _ => unreachable!(),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(samples: &'static [Sample]) -> Day {
        Day::text(1, |input| input.len().to_string(), |_| "42".into(), samples)
    }

    #[test]
    fn selected_part_does_not_require_other_part() {
        let day = fixture(&[sample!("length", "hello", part1 = "5")]);
        assert_eq!(check_samples(&day, Some(1), true), Ok(1));
        assert!(check_samples(&day, None, true)
            .unwrap_err()
            .contains("part 2: incomplete"));
    }

    #[test]
    fn samples_can_use_different_inputs_for_each_part() {
        let day = fixture(&[
            sample!("first", "hello", part1 = "5"),
            sample!("second", "", part2 = "42"),
        ]);
        assert_eq!(check_samples(&day, None, true), Ok(2));
    }

    #[test]
    fn both_expectations_and_compile_time_input_are_supported() {
        let day = fixture(&[sample!("both", "hello", part1 = "5", part2 = "42")]);
        assert_eq!(check_samples(&day, None, true), Ok(2));
        const MANIFEST: &str = input!("Cargo.toml");
        assert!(MANIFEST.contains("name = \"aoc\""));
    }

    #[test]
    fn wrong_answers_and_empty_samples_fail() {
        let day = fixture(&[sample!("wrong", "hello", part1 = "4")]);
        assert!(check_samples(&day, Some(1), true)
            .unwrap_err()
            .contains("got \"5\""));
        assert!(check_samples(&fixture(&[]), None, false).is_err());
        assert!(check_samples(&fixture(&[]), Some(2), true).is_err());
    }

    #[test]
    fn unfinished_solution_is_reported_as_failure() {
        let day = Day::text(
            1,
            |_| "0".into(),
            |_| todo!("part two"),
            &[sample!("unfinished", "", part2 = "42")],
        );
        assert!(check_samples(&day, Some(2), true)
            .unwrap_err()
            .contains("not yet implemented"));
    }

    #[test]
    fn invalid_native_commands_fail() {
        let days = [fixture(&[])];
        for args in [
            vec![],
            vec!["run", "1"],
            vec!["run", "99", "1"],
            vec!["test", "1", "3"],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(dispatch_result(&days, &args).is_err());
        }
    }
}
