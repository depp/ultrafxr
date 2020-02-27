use std::convert::TryFrom;
use std::str::{from_utf8, from_utf8_unchecked};

/// Parse a single Unicode code point from a possibly invalid stream of UTF-8
/// data. Returns the decoded character and the character length in bytes.
///
/// This will return the number of bytes for invalid sequences in the same way
/// that a web browser does when parsing UTF-8 HTML.
pub fn parse_character(text: &[u8]) -> (Option<char>, usize) {
    let c1 = match text.get(0) {
        None => return (None, 0),
        Some(&c) => c,
    };
    let (n, mincp, mut cp) = match c1 {
        0b00000000..=0b01111111 => return (Some(c1 as char), 1),
        0b11000000..=0b11011111 => (1, 1 << 7, c1 as u32 & 0b00011111),
        0b11100000..=0b11101111 => (2, 1 << 11, c1 as u32 & 0b00001111),
        0b11110000..=0b11110111 => (3, 1 << 16, c1 as u32 & 0b00000111),
        _ => return (None, 1),
    };
    for i in 0..n {
        match text.get(1 + i) {
            Some(&c) if 0b10000000 <= c && c <= 0b10111111 => {
                cp = (cp << 6) | (c as u32 & 0b00111111)
            }
            _ => return (None, i + 1),
        }
    }
    let c = if cp < mincp {
        None
    } else {
        char::try_from(cp).ok()
    };
    (c, n + 1)
}

/// Iterator over segments of UTF-8 text that may contain errors.
pub struct UTF8Segments<'a>(pub &'a [u8]);

/// A segment of valid or invalid UTF-8 text.
pub type UTF8Segment<'a> = Result<&'a str, &'a [u8]>;

impl<'a> Iterator for UTF8Segments<'a> {
    type Item = UTF8Segment<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let text = self.0;
        if text.len() == 0 {
            None
        } else {
            Some(match from_utf8(text) {
                Ok(s) => {
                    self.0 = &text[s.len()..];
                    Ok(s)
                }
                Err(e) => {
                    let n = e.valid_up_to();
                    if n == 0 {
                        let (_, n) = parse_character(text);
                        self.0 = &text[n..];
                        Err(&text[..n])
                    } else {
                        self.0 = &text[n..];
                        Ok(unsafe { from_utf8_unchecked(&text[..n]) })
                    }
                }
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;

    #[test]
    fn parse_character_success() -> Result<(), TestFailure> {
        let cases: &'static [char] = &[
            '\0',
            'a',
            '\u{0080}',
            '\u{07ff}',
            '\u{0800}',
            '\u{d7ff}',
            '\u{e000}',
            '\u{ffff}',
            '\u{10000}',
            '\u{10ffff}',
        ];
        let mut tests = Tests::new();
        let mut s = String::with_capacity(4);
        for (n, &c) in cases.iter().enumerate() {
            s.clear();
            s.push(c);
            let out = parse_character(s.as_ref());
            let expect = (Some(c), s.len());
            if !tests.add(out == expect) {
                eprintln!("Test {} failed: input={}", n, c.escape_unicode());
                eprintln!("    Got:    {:?}", out);
                eprintln!("    Expect: {:?}", expect);
            }
        }
        tests.done()
    }

    #[test]
    fn parse_character_error() -> Result<(), TestFailure> {
        let cases: &'static [(&'static [u8], usize)] = &[
            // End of file.
            (b"", 0),
            // Continuation character.
            (b"\x80", 1),
            (b"\x90", 1),
            (b"\xbf", 1),
            (b"\x90\x00", 1),
            (b"\x90\x80", 1),
            (b"\x90\xc0", 1),
            (b"\x90\xe0", 1),
            (b"\x90\xf0", 1),
            (b"\x90\xff", 1),
            // Invalid two-byte, invalid continuation.
            (b"\xc2", 1),
            (b"\xc2\x00", 1),
            (b"\xc2\xc0", 1),
            (b"\xc2\xe0", 1),
            (b"\xc2\xf0", 1),
            (b"\xc2\xff", 1),
            // Invalid two-byte, too small.
            (b"\xc0\x80_", 2),
            (b"\xc1\xbf_", 2),
            // Invaild three-byte, invalid continuation.
            (b"\xe0", 1),
            (b"\xe0\x00", 1),
            (b"\xe0\xc0", 1),
            (b"\xe0\xe0", 1),
            (b"\xe0\xf0", 1),
            (b"\xe0\xff", 1),
            (b"\xe0\xa0\x00", 2),
            (b"\xe0\xa0\xc0", 2),
            (b"\xe0\xa0\xe0", 2),
            (b"\xe0\xa0\xf0", 2),
            (b"\xe0\xa0\xff", 2),
            // Invalid three-byte, too small.
            (b"\xe0\x80\x80_", 3),
            (b"\xe0\x9f\xbf_", 3),
            // Invaild three-byte, surrogate.
            (b"\xed\xa0\x80_", 3),
            (b"\xed\xbf\xbf_", 3),
            // Invalid four-byte, invalid continuation.
            (b"\xf0", 1),
            (b"\xf0\x00", 1),
            (b"\xf0\xc0", 1),
            (b"\xf0\xe0", 1),
            (b"\xf0\xf0", 1),
            (b"\xf0\xff", 1),
            (b"\xf0\x90\x00", 2),
            (b"\xf0\x90\xc0", 2),
            (b"\xf0\x90\xe0", 2),
            (b"\xf0\x90\xf0", 2),
            (b"\xf0\x90\xff", 2),
            (b"\xf0\x90\x80\x00", 3),
            (b"\xf0\x90\x80\xc0", 3),
            (b"\xf0\x90\x80\xe0", 3),
            (b"\xf0\x90\x80\xf0", 3),
            (b"\xf0\x90\x80\xff", 3),
            // Invalid four-byte, too small.
            (b"\xf0\x80\x80\x80_", 4),
            (b"\xf0\x8f\xbf\xbf_", 4),
            // Invalid four-byte, too large.
            (b"\xf4\x90\x80\x80_", 4),
            (b"\xf7\xbf\xbf\xbf_", 4),
            // Invalid bytes.
            (b"\xf8\x80", 1),
            (b"\xff\x80", 1),
        ];
        let mut tests = Tests::new();
        for (n, &(input, size)) in cases.iter().enumerate() {
            let out = parse_character(input);
            let expect = (None, size);
            if !tests.add(out == expect) {
                eprintln!("Test {} failed: input={}", n, Str(input));
                eprintln!("    Got:    {:?}", out);
                eprintln!("    Expect: {:?}", expect);
            }
        }
        tests.done()
    }

    #[test]
    fn segments() -> Result<(), TestFailure> {
        let input: &'static [u8] = b"abc \x80 def \xe0\xa0\xf0";
        let expect: &'static [UTF8Segment<'static>] = &[
            Ok("abc "),
            Err(b"\x80"),
            Ok(" def "),
            Err(b"\xe0\xa0"),
            Err(b"\xf0"),
        ];
        let output: Vec<UTF8Segment<'static>> = UTF8Segments(input).collect();
        if output != expect {
            eprintln!("Failed");
            eprintln!("    Got:    {:?}", output);
            eprintln!("    Expect: {:?}", expect);
            return Err(TestFailure);
        }
        Ok(())
    }
}
