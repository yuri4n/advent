use std::{collections::HashMap, error::Error, fmt};
use utils::{InputBuilder, SolverBuilder};

fn main() {
    let input = InputBuilder::new(String::from("modules/y21/d08"), 26)
        .second_ans(61229)
        .build();

    let digit_segments: HashMap<u8, u8> = HashMap::from([(1, 2), (4, 4), (7, 3), (8, 7)]);
    let to_find = [1, 4, 7, 8];
    let lens_to_find: Vec<u8> = to_find
        .iter()
        .map(|n| *digit_segments.get(n).unwrap())
        .collect();

    let solver = SolverBuilder::new(input, |input| first(input, &lens_to_find))
        .second_fn(second)
        .build();

    println!("{:?}", solver.solve());
}

fn first(input: String, lens_to_find: &[u8]) -> i32 {
    input.lines().fold(0, |count, line| {
        line.split('|')
            .map(|part| part.split_whitespace())
            .last()
            .unwrap()
            .fold(0, |c, w| {
                c + (lens_to_find.contains(&(w.len() as u8)) as i32)
            })
            + count
    })
}

#[derive(Clone, Copy, Debug)]
struct InvalidIndex(usize);

impl fmt::Display for InvalidIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The index {} fell out of range.", self.0)
    }
}

impl Error for InvalidIndex {}

type Ob<'a> = &'a [char];

type DisplayInf<'a> = (usize, &'a [char]);

trait Digits<'a> {
    fn add(&mut self, display_inf: DisplayInf<'a>) -> Result<(), Box<dyn Error>>;
}

impl<'a> Digits<'a> for [Ob<'a>; 10] {
    fn add(&mut self, display_inf: DisplayInf<'a>) -> Result<(), Box<dyn Error>> {
        let (sym, segments) = display_inf;
        if let Some(el) = self.get_mut(sym) {
            *el = segments;
            Ok(())
        } else {
            Err(Box::new(InvalidIndex(sym)))
        }
    }
}

/// The number of segments needed to print the digit `i`.
const NEEDED_SEGMENTS: [usize; 10] = [6, 2, 5, 5, 4, 5, 6, 3, 7, 6];

/// '1' and '4' have one segment in common and one not.
const USEFUL: [usize; 2] = [1, 4];

fn second(input: String) -> i32 {
    let iter = input.lines().map(|line| {
        let mut parts = line.splitn(2, '|').map(|s| {
            s.split_whitespace()
                .map(|w| w.trim().chars().collect())
                .collect::<Vec<Vec<char>>>()
        });

        let (obs, display) = (parts.next().unwrap(), parts.next().unwrap());
        let mut digits: [&[char]; 10] = [&[]; 10];
        for u in USEFUL {
            let ns = NEEDED_SEGMENTS[u];
            digits
                .add((u, &(obs.iter().find(|ob| ob.len() == ns).unwrap())[..]))
                .unwrap();
        }

        // let unique = NEEDED_SEGMENTS.group_by(||)
        for digit in display {}

        println!("digits {:?}", digits);
    });

    iter.for_each(drop);

    0
}
