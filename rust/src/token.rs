use crate::sourcepos::{HasPos, Pos, Span};
use crate::utf8::parse_character;
use std::error::Error;
use std::fmt;

/// Tokenizer error. Not used for syntax errors.
#[derive(Debug, Clone, Copy)]
pub enum TokenError {
    TooMuchText,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenError::*;
        f.write_str(match self {
            TooMuchText => "too much text, exceeds 4 GiB maximum total input size",
        })
    }
}

impl Error for TokenError {}

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
    pos: u32,
    start_pos: u32,
}

// Return true if the character is ASCII whitespace.
fn is_space(c: u8) -> bool {
    // space, \t, \n, \v, \f, \r
    c == 32 || (9 <= c && c <= 13)
}

// Return true if the character can appear in a normal symbol.
fn is_symbol(c: u8) -> bool {
    match c as char {
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

// Return true if the character is a line break character, CR or LF.
fn is_line_break(c: u8) -> bool {
    c == b'\n' || c == b'\r'
}

/// Get the number of symbol characters at the beginning of a string.
fn symbol_len(text: &[u8]) -> usize {
    match text.iter().position(|&c| !is_symbol(c)) {
        Some(idx) => idx,
        _ => text.len(),
    }
}

impl<'a> Tokenizer<'a> {
    // Create a new tokenizer that returns a stream of tokens from the given text.
    pub fn new(text: &'a [u8]) -> Result<Self, TokenError> {
        let start_pos: u32 = 1;
        if text.len() > (u32::max_value() - start_pos) as usize {
            return Err(TokenError::TooMuchText);
        }
        Ok(Tokenizer {
            text,
            pos: 0,
            start_pos,
        })
    }

    /// Rewind tokenizer to start of stream.
    pub fn rewind(&mut self) -> () {
        self.pos = 0;
    }

    // Return the next token from the stream.
    pub fn next(&mut self) -> Token<'a> {
        use Type::*;
        let pos = match self.text[self.pos as usize..]
            .iter()
            .position(|&c| !is_space(c))
        {
            Some(n) => self.pos as usize + n,
            None => {
                let pos = self.text.len() as u32;
                self.pos = pos;
                return Token {
                    ty: End,
                    pos: Pos(pos),
                    text: &[],
                };
            }
        };
        let (&first, rest) = self.text[pos..].split_first().unwrap();
        let (ty, len) = match first as char {
            // Lower case
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm'
                | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' |
            // Upper case
            'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M'
                | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' |
            // Punctuation
            '!' | '$' | '%' | '&' | '*' | '/' | ':' | '<'
                | '=' | '>' | '?' | '@' | '^' | '_' | '~' =>
                (Symbol, symbol_len(rest)),
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' =>
                (Number, symbol_len(rest)),
            ';' =>
		(Comment, rest.iter().position(|&c| is_line_break(c)).unwrap_or(rest.len())),
            '.' => {
                // Either a number or a symbol.
                let ty = match rest.split_first() {
                    Some((&c, _)) if b'0' <= c && c <= b'9' => Number,
                    _ => Symbol,
                };
                (ty, symbol_len(rest))
            }
            '-' | '+' => {
                // Either a number or a symbol.
                let ty = match rest.split_first() {
                    Some((&c, rest)) => {
                        if b'0' <= c && c <= b'9' {
                            Number
                        } else if c == b'.' {
                            match rest.split_first() {
                                Some((&c, _)) if b'0' <= c && c <= b'9' => Number,
                                _ => Symbol,
                            }
                        } else {
                            Symbol
                        }
                    }
                    _ => Symbol,
                };
		(ty, symbol_len(rest))
            }
            '(' => (ParenOpen, 0),
            ')' => (ParenClose, 0),
            _ => {
		let (_, n) = parse_character(&self.text[pos..]);
		(Error, n-1)
	    }
        };
        let end = pos + 1 + len;
        self.pos = end as u32;
        Token {
            ty,
            pos: Pos(pos as u32 + self.start_pos),
            text: &self.text[pos..end],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, Tokenizer, Type};
    use crate::sourcepos::Pos;
    use crate::test::*;
    use std::fmt;

    fn tok_eq(x: &Token, y: &Token) -> bool {
        x.ty == y.ty
            && x.pos == y.pos
            && x.text.as_ptr() == y.text.as_ptr()
            && x.text.len() == y.text.len()
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
            (b"\xc2\x80 ", Error),
        ];
        let mut tests = Tests::new();
        for (n, &(input, ty)) in cases.iter().enumerate() {
            let baretok = &input[..input.len() - 1];
            let etok = Token {
                ty,
                pos: Pos(1),
                text: &input[..input.len() - 1],
            };
            for input in [baretok, input].iter() {
                let mut toks = match Tokenizer::new(input) {
                    Ok(toks) => toks,
                    Err(e) => {
                        eprintln!("Test {} failed: input={}", n, Str(input));
                        eprintln!("    Error: {}", e);
                        tests.add(false);
                        continue;
                    }
                };
                let tok = toks.next();
                if !tests.add(tok_eq(&tok, &etok)) {
                    eprintln!("Test {} failed: input={}", n, Str(input));
                    eprintln!("    Got:    {}", Tok(&tok));
                    eprintln!("    Expect: {}", Tok(&etok));
                }
            }
        }
        tests.done()
    }
}
