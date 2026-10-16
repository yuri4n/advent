use aoc::winnow::ascii::dec_int;

pub struct Solver;
impl aoc::Solution for Solver {
    type Prepared<'a> = Vec<i32>;
    type Answer = i32;
    const PARTS: &'static [u8] = &[2];
    fn snapshot(input: Vec<i32>) -> Result<Box<dyn Fn() -> Vec<i32>>, String> {
        Ok(Box::new(move || input.clone()))
    }

    fn process(input: aoc::Input<'_>) -> Result<Vec<i32>, String> {
        let mut calories = vec![0];
        for value in aoc::records(input.lines(), |line| {
            if line.is_empty() { Ok(None) } else { aoc::parse(line, dec_int::<_, i32, _>).map(Some) }
        }) {
            match value? {
                Some(value) => *calories.last_mut().unwrap() += value,
                None => calories.push(0),
            }
        }
        Ok(calories)
    }
    fn part1(_input: Vec<i32>) -> Result<i32, String> {
        todo!("2022 day 1 part 1")
    }
    fn part2(mut calories: Vec<i32>) -> Result<i32, String> {
        calories.sort();
        calories.reverse();
        Ok(calories.iter().take(3).sum())
    }
}

pub const EXAMPLES: &[aoc::Example<i32>] = &[aoc::example!(
    "example",
    "1000\n2000\n3000\n\n4000\n\n5000\n6000\n\n7000\n8000\n9000\n\n10000",
    part2 = 45000
)];
