//! Small, dependency-free Advent of Code harness.
//!
//! A year registers normal Rust modules with [`main!`]. Each module exports
//! `part1(&str)`, `part2(&str)` (any `Display` answer), and `SAMPLES`.

use std::io::{self, Read};

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub name: &'static str,
    pub input: &'static str,
    pub part1: Option<&'static str>,
    pub part2: Option<&'static str>,
}

pub struct Day {
    pub number: u8,
    pub part1: fn(&str) -> String,
    pub part2: fn(&str) -> String,
    pub samples: &'static [Sample],
}

/// Define an inline sample with one or both expected answers, written as strings.
/// Omit an expected answer while its part is unfinished.
#[macro_export]
macro_rules! sample {
    ($name:expr, $input:expr, part1 = $first:expr, part2 = $second:expr $(,)?) => {
        $crate::Sample {
            name: $name,
            input: $input,
            part1: Some($first),
            part2: Some($second),
        }
    };
    ($name:expr, $input:expr, part1 = $first:expr $(,)?) => {
        $crate::Sample {
            name: $name,
            input: $input,
            part1: Some($first),
            part2: None,
        }
    };
    ($name:expr, $input:expr, part2 = $second:expr $(,)?) => {
        $crate::Sample {
            name: $name,
            input: $input,
            part1: None,
            part2: Some($second),
        }
    };
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
    ($($number:literal => $module:ident),+ $(,)?) => {
        $(mod $module;)+

        fn main() {
            let days = &[$($crate::Day {
                number: $number,
                part1: |input| $module::part1(input).to_string(),
                part2: |input| $module::part2(input).to_string(),
                samples: $module::SAMPLES,
            }),+];
            std::process::exit($crate::dispatch(days, std::env::args().skip(1)));
        }

        #[cfg(test)]
        mod inline_samples {
            $(#[test]
            fn $module() {
                let day = $crate::Day {
                    number: $number,
                    part1: |input| super::$module::part1(input).to_string(),
                    part2: |input| super::$module::part2(input).to_string(),
                    samples: super::$module::SAMPLES,
                };
                // Native tests validate the expectations that have been supplied.
                // `aoc test` additionally requires coverage for the requested parts.
                $crate::check_samples(&day, None, false).unwrap();
            })+
        }
    };
}

fn solve(day: &Day, part: u8, input: &str) -> Result<String, String> {
    let solver = match part {
        1 => day.part1,
        2 => day.part2,
        _ => return Err("part must be 1 or 2".into()),
    };
    std::panic::catch_unwind(|| solver(input)).map_err(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("solution panicked");
        format!("day {} part {part}: {message}", day.number)
    })
}

/// Validate provided expectations. Strict mode also requires every selected part
/// to have at least one expectation; missing coverage never counts as a pass.
pub fn check_samples(day: &Day, part: Option<u8>, strict: bool) -> Result<usize, String> {
    if part.is_some_and(|part| part != 1 && part != 2) {
        return Err("part must be 1 or 2".into());
    }
    let mut checked = [0usize; 2];
    let mut failures = Vec::new();
    for sample in day.samples {
        for current in 1..=2 {
            if part.is_some_and(|part| part != current) {
                continue;
            }
            let expected = if current == 1 {
                sample.part1
            } else {
                sample.part2
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
    if !(args.len() == 2 || args.len() == 3) || !matches!(args[0].as_str(), "run" | "test") {
        return Err("usage: <year-binary> run DAY PART | test DAY [1|2|both] | list".into());
    }
    let number = args[1].parse::<u8>().map_err(|_| "invalid day")?;
    let day = days
        .iter()
        .find(|day| day.number == number)
        .ok_or_else(|| format!("day {number} is not registered"))?;
    let part = match args.get(2).map(String::as_str).unwrap_or("both") {
        "1" => Some(1),
        "2" => Some(2),
        "both" => None,
        _ => return Err("part must be 1, 2 or both".into()),
    };
    match args[0].as_str() {
        "run" => {
            let part = part.ok_or("run requires part 1 or 2")?;
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            println!("{}", solve(day, part, &input)?);
        }
        "test" => {
            let count = check_samples(day, part, true)?;
            println!("day {number}: {count} inline sample expectation(s) passed");
        }
        _ => unreachable!(),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(samples: &'static [Sample]) -> Day {
        Day {
            number: 1,
            part1: |input| input.len().to_string(),
            part2: |_| "42".into(),
            samples,
        }
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
        let day = Day {
            part2: |_| todo!("part two"),
            ..fixture(&[sample!("unfinished", "", part2 = "42")])
        };
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
