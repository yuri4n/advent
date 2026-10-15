pub fn part1(_input: &str) -> usize {
    todo!("2021 day 7 part 1")
}

pub fn part2(input: &str) -> usize {
    let xs = input.trim().split(',').map(|x| x.parse::<usize>().unwrap());
    let max = xs.clone().max().unwrap();
    let mut positions = vec![0usize; max + 1];

    for x in xs {
        positions[x] += 1;
    }

    let mut min_to_pos = vec![0usize; max + 1];
    for (i, el) in min_to_pos.iter_mut().enumerate() {
        let mut for_each_pos = vec![0usize; max + 1];
        for (j, y) in positions.iter().enumerate() {
            for_each_pos[j] = y * com(j.abs_diff(i));
        }
        *el = for_each_pos.iter().sum();
    }
    *min_to_pos.iter().min().unwrap()
}

fn com(i: usize) -> usize {
    (i * (i + 1)) / 2
}

pub const SAMPLES: &[aoc::Sample] = &[
    aoc::sample!("example", "16,1,2,0,4,2,7,1,2,14", part2 = "168"),
];
