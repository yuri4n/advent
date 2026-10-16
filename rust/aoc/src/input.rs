use std::{
    cell::RefCell,
    io::{self, BufRead, BufReader, Read},
    rc::Rc,
};

pub type Chars<'a> = Box<dyn Iterator<Item = io::Result<char>> + 'a>;
pub type LineChars<'a> = Box<dyn Iterator<Item = io::Result<Chars<'a>>> + 'a>;

/// A byte-backed input. Selecting a view consumes this handle, not the source
/// eagerly. The runner retains ownership of borrowed file/stdin handles.
pub struct Input<'a> {
    reader: Box<dyn BufRead + 'a>,
}
impl<'a> Input<'a> {
    pub fn new(source: impl Read + 'a) -> Self {
        Self {
            reader: Box::new(BufReader::with_capacity(4096, source)),
        }
    }
    pub fn bytes(self) -> Box<dyn Iterator<Item = io::Result<u8>> + 'a> {
        Box::new(self.reader.bytes())
    }
    pub fn text(mut self) -> io::Result<String> {
        let mut text = String::new();
        self.reader.read_to_string(&mut text)?;
        Ok(text)
    }
    pub fn chars(self) -> Chars<'a> {
        let mut bytes = self.reader.bytes();
        let mut done = false;
        Box::new(std::iter::from_fn(move || {
            if done {
                return None;
            }
            let first = match bytes.next()? {
                Ok(b) => b,
                Err(e) => {
                    done = true;
                    return Some(Err(e));
                }
            };
            let width = match first {
                0..=0x7f => 1,
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => 0,
            };
            let mut buffer = [first, 0, 0, 0];
            if width == 0 {
                done = true;
                return Some(Err(invalid_utf8()));
            }
            for slot in buffer.iter_mut().take(width).skip(1) {
                match bytes.next() {
                    Some(Ok(b)) => *slot = b,
                    Some(Err(e)) => {
                        done = true;
                        return Some(Err(e));
                    }
                    None => {
                        done = true;
                        return Some(Err(invalid_utf8()));
                    }
                }
            }
            match std::str::from_utf8(&buffer[..width]) {
                Ok(text) => Some(Ok(text.chars().next().unwrap())),
                Err(_) => {
                    done = true;
                    Some(Err(invalid_utf8()))
                }
            }
        }))
    }
    /// Borrowed inner iterators expire when the outer iterator advances. Any
    /// unread remainder is discarded without allocating a complete line.
    pub fn line_chars(self) -> LineChars<'a> {
        let state = Rc::new(RefCell::new(Cursor {
            chars: self.chars(),
            peek: None,
            eof: false,
            ended: true,
            generation: 0,
        }));
        Box::new(std::iter::from_fn(move || {
            let mut cursor = state.borrow_mut();
            while !cursor.ended {
                if let Err(e) = cursor.line_char() {
                    return Some(Err(e));
                }
            }
            cursor.generation += 1;
            match cursor.peek() {
                Ok(None) => return None,
                Err(e) => return Some(Err(e)),
                _ => {}
            }
            cursor.ended = false;
            let id = cursor.generation;
            drop(cursor);
            let line = state.clone();
            let mut ended = false;
            let inner = std::iter::from_fn(move || {
                if ended {
                    return None;
                }
                let mut cursor = line.borrow_mut();
                if id != cursor.generation {
                    ended = true;
                    return Some(Err(io::Error::other(
                        "line view expired; consume it before advancing the outer iterator",
                    )));
                }
                match cursor.line_char() {
                    Ok(Some(c)) => Some(Ok(c)),
                    Ok(None) => {
                        ended = true;
                        None
                    }
                    Err(e) => {
                        ended = true;
                        Some(Err(e))
                    }
                }
            });
            Some(Ok(Box::new(inner) as Chars<'a>))
        }))
    }
    pub fn lines(self) -> crate::Lines<'a> {
        Box::new(self.line_chars().map(|line| line?.collect()))
    }
}
fn invalid_utf8() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid or incomplete UTF-8 input",
    )
}
struct Cursor<'a> {
    chars: Chars<'a>,
    peek: Option<char>,
    eof: bool,
    ended: bool,
    generation: u64,
}
impl Cursor<'_> {
    fn peek(&mut self) -> io::Result<Option<char>> {
        if self.peek.is_none() && !self.eof {
            match self.chars.next() {
                Some(Ok(c)) => self.peek = Some(c),
                Some(Err(e)) => {
                    self.eof = true;
                    self.ended = true;
                    return Err(e);
                }
                None => self.eof = true,
            }
        }
        Ok(self.peek)
    }
    fn take(&mut self) -> io::Result<Option<char>> {
        self.peek()?;
        Ok(self.peek.take())
    }
    fn line_char(&mut self) -> io::Result<Option<char>> {
        if self.ended {
            return Ok(None);
        }
        match self.take()? {
            Some('\r') => {
                if self.peek()? == Some('\n') {
                    self.take()?;
                }
                self.ended = true;
                Ok(None)
            }
            Some('\n') | None => {
                self.ended = true;
                Ok(None)
            }
            value => Ok(value),
        }
    }
}
