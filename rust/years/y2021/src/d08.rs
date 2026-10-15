use std::collections::HashMap;

pub fn part1(input: &str) -> i32 {
    let digit_segments: HashMap<u8, u8> = HashMap::from([(1, 2), (4, 4), (7, 3), (8, 7)]);
    let to_find = [1, 4, 7, 8];
    let lens_to_find: Vec<u8> = to_find
        .iter()
        .map(|n| *digit_segments.get(n).unwrap())
        .collect();

    input.lines().fold(0, |count, line| {
        line.split('|')
            .map(|part| part.split_whitespace())
            .next_back()
            .unwrap()
            .fold(0, |c, w| {
                c + (lens_to_find.contains(&(w.len() as u8)) as i32)
            })
            + count
    })
}

pub fn part2(_input: &str) -> i32 {
    todo!("2021 day 8 part 2; known example answer: 61229")
}

pub const SAMPLES: &[aoc::Sample] = &[aoc::sample!(
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
    part1 = "26"
)];
