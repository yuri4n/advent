use std::{
    io::Cursor,
    sync::atomic::{AtomicUsize, Ordering},
};
static PARSED: AtomicUsize = AtomicUsize::new(0);

struct Numbers;
impl aoc::Solution for Numbers {
    type Prepared<'a> = aoc::Records<'a, i64>;
    type Answer = i64;
    const PARTS: &'static [u8] = &[1, 2];
    fn snapshot(
        values: Self::Prepared<'_>,
    ) -> Result<Box<dyn Fn() -> Self::Prepared<'static>>, String> {
        let values: Vec<i64> = values.collect::<Result<_, _>>()?;
        Ok(Box::new(move || {
            Box::new(values.clone().into_iter().map(Ok))
        }))
    }

    fn process(input: aoc::Input<'_>) -> Result<Self::Prepared<'_>, String> {
        Ok(aoc::records(input.lines(), |line| {
            PARSED.fetch_add(1, Ordering::SeqCst);
            aoc::parse(line, aoc::winnow::ascii::dec_int)
        }))
    }
    fn part1(mut values: Self::Prepared<'_>) -> Result<i64, String> {
        values.next().ok_or("empty input")?
    }
    fn part2(mut values: Self::Prepared<'_>) -> Result<i64, String> {
        values.try_fold(0, |sum, value| value.map(|value| sum + value))
    }
}

#[test]
fn processing_is_lazy_typed_and_repeated_for_each_execution() {
    let day = aoc::Day::processed::<Numbers>(1, &[]);
    assert_eq!(day.solve(1, &mut Cursor::new("7\r\nbad\n")), Ok("7".into()));
    for _ in 0..2 {
        assert_eq!(day.solve(2, &mut Cursor::new("7\r\n3\n")), Ok("10".into()));
    }
    let error = day.solve(2, &mut Cursor::new("7\n3junk\n")).unwrap_err();
    assert!(error.contains("line 2"), "{error}");
    let before = PARSED.load(Ordering::SeqCst);
    let report = day.benchmark(2, "7\n3\n", 2, 1).unwrap();
    assert_eq!(PARSED.load(Ordering::SeqCst) - before, 8); // Three total passes + one snapshot; no solve-only parsing.
    assert_eq!(report.answer, "10");
    assert_eq!(report.timings.len(), 2);
}
