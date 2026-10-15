pub fn part1(_input: &str) -> i32 {
    todo!("2022 day 1 part 1")
}

pub fn part2(input: &str) -> i32 {
    let mut calories: Vec<i32> = input
        .split("\n\n")
        .map(|elf| elf.lines().map(|cal| cal.parse::<i32>().unwrap()).sum())
        .collect();
    calories.sort();
    calories.reverse();
    calories.iter().take(3).sum()
}

pub const SAMPLES: &[aoc::Sample] = &[aoc::sample!(
    "example",
    "1000\n2000\n3000\n\n4000\n\n5000\n6000\n\n7000\n8000\n9000\n\n10000",
    part2 = "45000"
)];
