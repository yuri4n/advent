use std::collections::HashMap;
use aoc::winnow::{ascii::{alpha1, space0, space1}, combinator::{delimited, separated, separated_pair}, Parser};

type Entry = (Vec<String>, Vec<String>);
pub struct Solver;
impl aoc::Solution for Solver {
    type Prepared<'a> = aoc::Records<'a, Entry>;
    type Answer = i32;
    const PARTS: &'static [u8] = &[1];
    fn snapshot(input: Self::Prepared<'_>) -> Result<Box<dyn Fn() -> Self::Prepared<'static>>, String> {
        let values: Vec<Entry> = input.collect::<Result<_, _>>()?;
        Ok(Box::new(move || Box::new(values.clone().into_iter().map(Ok))))
    }

    fn process(input: aoc::Input<'_>) -> Result<Self::Prepared<'_>, String> {
        Ok(aoc::records(input.lines(), |line| {
            let words = || separated(1.., alpha1.map(str::to_owned), space1);
            aoc::parse(line, separated_pair(words(), delimited(space0, "|", space0), words()))
        }))
    }
    fn part1(input: Self::Prepared<'_>) -> Result<i32, String> { part1(input) }
    fn part2(_input: Self::Prepared<'_>) -> Result<i32, String> {
        todo!("2021 day 8 part 2; known example answer: 61229")
    }
}

fn part1(mut input: aoc::Records<'_, Entry>) -> Result<i32, String> {
    let digit_segments: HashMap<u8, u8> = HashMap::from([(1, 2), (4, 4), (7, 3), (8, 7)]);
    let to_find = [1, 4, 7, 8];
    let lens_to_find: Vec<u8> = to_find
        .iter()
        .map(|n| *digit_segments.get(n).unwrap())
        .collect();

    input.try_fold(0, |count, entry| {
        let (_, output) = entry?;
        Ok(count + output.iter().fold(0, |c, word| {
            c + (lens_to_find.contains(&(word.len() as u8)) as i32)
        }))
    })
}

pub const EXAMPLES: &[aoc::Example<i32>] = &[aoc::example!(
    "example",
    r#"be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce"#,
    part1 = 26
)];
