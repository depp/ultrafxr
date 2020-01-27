use crate::sourcepos::{HasPos, Pos, Span};

// Token types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    End,   // End of input.
    Error, // Invalid character.
    Comment,
    Symbol,
    Number,
    ParenOpen,
    ParenClose,
}

// A token in an s-expression.
#[derive(Clone, Copy, Debug)]
pub struct Token<'a> {
    pub ty: Type,
    pub pos: Pos,
    pub text: &'a [u8],
}

impl HasPos for Token<'_> {
    fn source_pos(&self) -> Span {
        let Pos(off) = self.pos;
        Span {
            start: self.pos,
            end: Pos(off + self.text.len() as u32),
        }
    }
}

pub struct Tokenizer<'a> {
    text: &'a [u8],
    pos: Pos,
}

// Return true if the character is ASCII whitespace.
fn is_space(c: u8) -> bool {
    // space, \t, \n, \v, \f, \r
    c == 32 || (9 <= c && c <= 13)
}

// Return true if the character can appear in a normal symbol.
fn is_symbol(c: &u8) -> bool {
    match *c as char {
        // Lower case
        'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm'
            | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' |
        // Upper case
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M'
            | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' |
        // Digits
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' |
        // Punctuation
        '-' | '!' | '$' | '%' | '&' | '*' | '+' | '.' | '/' | ':'
            | '<' | '=' | '>' | '?' | '@' | '^' | '_' | '~' => true,
        _ => false,
    }
}

// Return true if the character cannot appear in a normal symbol.
fn is_not_symbol(c: &u8) -> bool {
    !is_symbol(c)
}

// Return true if the character is a line break character, CR or LF.
fn is_line_break(c: &u8) -> bool {
    *c == b'\n' || *c == b'\r'
}

// Drop the prefix of a slice that contains symbol characters, return the rest.
fn drop_symbol(text: &[u8]) -> &[u8] {
    match text.iter().position(is_not_symbol) {
        Some(idx) => &text[idx..],
        _ => text,
    }
}

impl<'a> Tokenizer<'a> {
    // Create a new tokenizer that returns a stream of tokens from the given text.
    pub fn new(text: &'a [u8]) -> Self {
        Tokenizer { text, pos: Pos(1) }
    }

    // Return the next token from the stream.
    pub fn next(&mut self) -> Token<'a> {
        use Type::*;
        let (first, mut rest) = loop {
            match self.text.split_first() {
                None => {
                    return Token {
                        ty: End,
                        pos: self.pos,
                        text: &[],
                    }
                }
                Some((&first, rest)) => {
                    if is_space(first) {
                        self.text = rest;
                        self.pos = match self.pos {
                            Pos(i) => Pos(i + 1),
                        };
                    } else {
                        break (first, rest);
                    }
                }
            }
        };
        let ty = match first as char {
            // Lower case
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm'
                | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' |
            // Upper case
            'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M'
                | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' |
            // Punctuation
            '!' | '$' | '%' | '&' | '*' | '/' | ':' | '<'
                | '=' | '>' | '?' | '@' | '^' | '_' | '~' => {
                rest = drop_symbol(rest);
                Symbol
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                rest = drop_symbol(rest);
                Number
            }
            ';' => {
                rest = match rest.iter().position(is_line_break) {
                    Some(idx) => &rest[idx..],
                    _ => &[],
                };
                Comment
            }
            '.' => {
                // Either a number or a symbol.
                let ty = match rest.split_first() {
                    Some((c, _)) if b'0' <= *c && *c <= b'9' => Number,
                    _ => Symbol,
                };
                rest = drop_symbol(rest);
                ty
            }
            '-' | '+' => {
                // Either a number or a symbol.
                let ty = match rest.split_first() {
                    Some((c, rest)) => {
                        if b'0' <= *c && *c <= b'9' {
                            Number
                        } else if *c == b'.' {
                            match rest.split_first() {
                                Some((c, _)) if b'0' <= *c && *c <= b'9' => Number,
                                _ => Symbol,
                            }
                        } else {
                            Symbol
                        }
                    }
                    _ => Symbol,
                };
                rest = drop_symbol(rest);
                ty
            }
            '(' => ParenOpen,
            ')' => ParenClose,
            _ => Error,
        };
        let len = self.text.len() - rest.len();
        let tok = Token {
            ty,
            pos: self.pos,
            text: &self.text[..len],
        };
        self.text = rest;
        self.pos = match self.pos {
            Pos(i) => Pos(i + len as u32),
        };
        tok
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, Tokenizer, Type};
    use crate::sourcepos::Pos;
    use std::error::Error;
    use std::fmt;

    fn tok_eq(x: &Token, y: &Token) -> bool {
        x.ty == y.ty
            && x.pos == y.pos
            && x.text.as_ptr() == y.text.as_ptr()
            && x.text.len() == y.text.len()
    }

    struct Str<'a>(&'a [u8]);

    impl<'a> fmt::Display for Str<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Str(b) = self;
            match std::str::from_utf8(b) {
                Ok(s) => fmt::Debug::fmt(s, f),
                _ => fmt::Debug::fmt(b, f),
            }
        }
    }

    struct Tok<'a>(&'a Token<'a>);

    impl<'a> fmt::Display for Tok<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Tok(&Token {
                ty,
                pos: Pos(pos),
                text,
            }) = self;
            write!(f, "pos={}, type={:?}, text={}", pos, ty, Str(text))
        }
    }

    #[derive(Debug)]
    struct TestFailure;

    impl fmt::Display for TestFailure {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "test failed")
        }
    }

    impl Error for TestFailure {
        fn description(&self) -> &str {
            "test failed"
        }
    }

    // An object which tracks the number of successful and failed tests.
    struct Tests {
        success: u32,
        failure: u32,
    }

    impl Tests {
        fn new() -> Tests {
            Tests {
                success: 0,
                failure: 0,
            }
        }
        fn add(&mut self, success: bool) -> bool {
            if success {
                self.success += 1;
            } else {
                self.failure += 1;
            }
            success
        }
        fn done(self) -> Result<(), TestFailure> {
            if self.failure > 0 {
                eprintln!(
                    "Error: {} of {} tests failed.",
                    self.failure,
                    self.success + self.failure
                );
                Err(TestFailure)
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_simple_tokens() -> Result<(), TestFailure> {
        use Type::*;
        let cases: &[(&'static [u8], Type)] = &[
            (b";comment\n", Comment),
            (b";\n", Comment),
            (b"symbol ", Symbol),
            (b"ABCXYZ ", Symbol),
            (b"ZYXCBA ", Symbol),
            (b"abcxyz ", Symbol),
            (b"zyxcba ", Symbol),
            (b"a0123456789 ", Symbol),
            (b"s;", Symbol),
            (b"s\n", Symbol),
            (b"s(", Symbol),
            (b"s)", Symbol),
            (b". ", Symbol),
            (b"- ", Symbol),
            (b"+ ", Symbol),
            (b"-. ", Symbol),
            (b"+. ", Symbol),
            (b"0 ", Number),
            (b"987 ", Number),
            (b"5.0abc@@&* ", Number),
            (b"+0 ", Number),
            (b"+555 ", Number),
            (b"-9 ", Number),
            (b".00 ", Number),
            (b".99 ", Number),
            (b".67 ", Number),
            (b"-.0 ", Number),
            (b"+.9 ", Number),
            (b"(a", ParenOpen),
            (b")a", ParenClose),
            (b"\x01 ", Error),
            (b"\x7f ", Error),
            (b"\x80 ", Error),
            (b"\xff ", Error),
        ];
        let mut tests = Tests::new();
        for (n, (input, ty)) in cases.iter().enumerate() {
            let tok = Tokenizer::new(input).next();
            let etok = Token {
                ty: *ty,
                pos: Pos(1),
                text: &input[..input.len() - 1],
            };
            if !tests.add(tok_eq(&tok, &etok)) {
                eprintln!("Test {} failed: input={}", n, Str(input));
                eprintln!("    Got:    {}", Tok(&tok));
                eprintln!("    Expect: {}", Tok(&etok));
            }
        }
        tests.done()
    }
}
