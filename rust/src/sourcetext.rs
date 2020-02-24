use crate::sourcepos::{Pos, Span};

// A decoded position within a source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextPos {
    // Line number, 0 indexed.
    pub line: u32,
    // Byte offset within line.
    pub byte: u32,
}

// A decoder for source positions within a single source file.
pub struct SourceText<'a> {
    text: &'a [u8],
    lines: Vec<u32>, // Start offset of each line.
    span: Span,
}

impl<'a> SourceText<'a> {
    // Create a new source location decoder for a file with the given contents.
    pub fn new(text: &'a [u8]) -> Self {
        let mut prev = b'\0';
        let mut lines = Vec::<u32>::new();
        lines.push(0);
        for (n, &c) in text.iter().enumerate() {
            match c {
                b'\n' => {
                    if prev == b'\r' {
                        lines.pop();
                    }
                    lines.push(n as u32 + 1);
                }
                b'\r' => {
                    lines.push(n as u32 + 1);
                }
                _ => {}
            }
            prev = c;
        }
        SourceText {
            text,
            lines,
            span: Span {
                start: Pos(1),
                end: Pos(text.len() as u32 + 1),
            },
        }
    }

    // Convert a byte offset to a line number and character offset.
    pub fn lookup(&self, pos: Pos) -> Option<TextPos> {
        if pos < self.span.start || self.span.end < pos {
            return None;
        }
        let offset = pos.0 - self.span.start.0;
        Some(match self.lines.binary_search(&offset) {
            Ok(i) => TextPos {
                line: i as u32,
                byte: 0,
            },
            Err(i) => TextPos {
                line: (i - 1) as u32,
                byte: offset - self.lines[i - 1],
            },
        })
    }

    // Get the contents of the given zero-indexed line. The line break is not
    // included.
    pub fn line(&self, index: u32) -> &'a [u8] {
        let i = index as usize;
        if i >= self.lines.len() {
            return &[];
        } else if i + 1 == self.lines.len() {
            &self.text[self.lines[i] as usize..]
        } else {
            let line = &self.text[self.lines[i] as usize..self.lines[i + 1] as usize];
            match line.split_last() {
                Some((b'\n', line)) => match line.split_last() {
                    Some((b'\r', line)) => line,
                    _ => line,
                },
                Some((b'\r', line)) => line,
                _ => line,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{SourceText, TextPos};
    use crate::sourcepos::Pos;

    fn test_lookup(input: &[u8], outputs: &[(u32, u32)]) {
        assert_eq!(input.len() + 1, outputs.len());
        let text = SourceText::new(input);
        let mut success = true;
        for (n, &expect) in (1..).zip(outputs.iter()) {
            let expect = Some(match expect {
                (line, byte) => TextPos { line, byte },
            });
            let result = text.lookup(Pos(n));
            if result != expect {
                success = false;
                eprintln!("Lookup failed: input={:?}, pos={}", input, n);
                eprintln!("    Got    {:?}", result);
                eprintln!("    Expect {:?}", expect);
            }
        }
        for &offset in [0, input.len() as u32 + 2].iter() {
            if let Some(result) = text.lookup(Pos(offset)) {
                success = false;
                eprintln!("lookup({}): got {:?}, expect None", offset, result);
            }
        }
        assert!(success);
    }

    #[test]
    fn test_lookup_simple() {
        test_lookup(
            b"ab\ncd\n",
            &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (2, 0)],
        );
    }

    #[test]
    fn test_lookup_no_linebreak_at_end() {
        test_lookup(
            b"abc\n\nd",
            &[(0, 0), (0, 1), (0, 2), (0, 3), (1, 0), (2, 0), (2, 1)],
        );
    }

    #[test]
    fn test_lookup_empty() {
        test_lookup(&[], &[(0, 0)]);
    }

    #[test]
    fn test_lookup_crlf() {
        test_lookup(
            b"a\r\nb\r\n",
            &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (2, 0)],
        );
    }

    #[test]
    fn test_lookup_cr() {
        test_lookup(b"a\rb\r", &[(0, 0), (0, 1), (1, 0), (1, 1), (2, 0)]);
    }

    #[test]
    fn test_line() {
        let text = SourceText::new(b"abc\ndef\rghi\r\njkl");
        let lines: &[&'static [u8]] = &[b"abc", b"def", b"ghi", b"jkl"];
        let mut success = true;
        for (n, &line) in lines.iter().enumerate() {
            let got = text.line(n as u32);
            if got != line {
                success = false;
                eprintln!("Line {}: got {:?}, expect {:?}", n, got, line);
            }
        }
        assert!(success);
    }
}
