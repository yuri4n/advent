use aoc::Input;
use std::{
    cell::Cell,
    io::{self, Cursor, Read},
    rc::Rc,
};

struct Counted {
    bytes: Cursor<Vec<u8>>,
    count: Rc<Cell<usize>>,
}
impl Read for Counted {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let n = self.bytes.read(out)?;
        self.count.set(self.count.get() + n);
        Ok(n)
    }
}
struct ByteCount;
impl aoc::Solution for ByteCount {
    type Prepared<'a> = Vec<u8>;
    type Answer = usize;
    const PARTS: &'static [u8] = &[1];
    fn process(input: Input<'_>) -> Result<Vec<u8>, String> {
        input
            .bytes()
            .collect::<io::Result<_>>()
            .map_err(|e| e.to_string())
    }
    fn part1(bytes: Vec<u8>) -> Result<usize, String> {
        Ok(bytes.len())
    }
    fn snapshot(bytes: Vec<u8>) -> Result<Box<dyn Fn() -> Vec<u8>>, String> {
        Ok(Box::new(move || bytes.clone()))
    }
}
#[test]
fn byte_benchmarks_do_not_decode_utf8() {
    let day = aoc::Day::processed::<ByteCount>(1, &[]);
    assert_eq!(
        day.benchmark_bytes(1, &[255, 254], 2, 1).unwrap().answer,
        "2"
    );
}

#[test]
fn byte_input_decodes_and_streams_nested_lines_without_collecting_them() {
    let text = "é雪🙂\r\n\nlast";
    assert_eq!(
        Input::new(Cursor::new(text))
            .chars()
            .collect::<io::Result<String>>()
            .unwrap(),
        text
    );
    assert_eq!(
        Input::new(Cursor::new(text))
            .lines()
            .collect::<io::Result<Vec<_>>>()
            .unwrap(),
        vec!["é雪🙂", "", "last"]
    );
    let count = Rc::new(Cell::new(0));
    let data = format!("{}\nnext", "a".repeat(100000)).into_bytes();
    let source = Counted {
        bytes: Cursor::new(data),
        count: count.clone(),
    };
    let mut lines = Input::new(source).line_chars();
    assert_eq!(
        lines
            .next()
            .unwrap()
            .unwrap()
            .take(2)
            .collect::<io::Result<String>>()
            .unwrap(),
        "aa"
    );
    assert!(count.get() <= 4096, "read entire line: {}", count.get());
    let values = Input::new(Cursor::new("abcdef\r\nxyz\n"))
        .line_chars()
        .map(|line| line?.take(2).collect::<io::Result<String>>())
        .collect::<io::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(values, vec!["ab", "xy"]);
    assert!(Input::new(Cursor::new([0xff]))
        .chars()
        .next()
        .unwrap()
        .is_err());
}
