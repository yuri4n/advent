use aoc::winnow::{ascii::dec_uint, combinator::separated};

pub struct Solver;
impl aoc::Solution for Solver {
    type Prepared<'a> = Vec<usize>;
    type Answer = usize;
    const PARTS: &'static [u8] = &[2];
    fn snapshot(input: Vec<usize>) -> Result<Box<dyn Fn() -> Vec<usize>>, String> {
        Ok(Box::new(move || input.clone()))
    }

    fn process(input: aoc::Input<'_>) -> Result<Vec<usize>, String> {
        let rows: Vec<Vec<usize>> = aoc::records(input.lines(), |line| {
            aoc::parse(line.trim(), separated(1.., dec_uint::<_, usize, _>, ","))
        }).collect::<Result<_, _>>()?;
        Ok(rows.into_iter().flatten().collect())
    }
    fn part1(_input: Vec<usize>) -> Result<usize, String> {
        todo!("2021 day 7 part 1")
    }
    fn part2(input: Vec<usize>) -> Result<usize, String> { Ok(part2(input)) }
}

fn part2(xs: Vec<usize>) -> usize {
    let max = *xs.iter().max().unwrap();
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

pub const EXAMPLES: &[aoc::Example<usize>] = &[
    aoc::example!("example", "16,1,2,0,4,2,7,1,2,14", part2 = 168),
];
