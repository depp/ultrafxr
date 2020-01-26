use crate::sourcepos::Pos;

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

pub struct Tokenizer<'a> {
    text: &'a [u8],
    pos: Pos,
}

fn is_space(c: u8) -> bool {
    // space, \t, \n, \v, \f, \r
    c == 32 || (9 <= c && c <= 13)
}

impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a [u8]) -> Self {
        Tokenizer { text, pos: Pos(1) }
    }

    pub fn next(&mut self) -> Token<'a> {
        let (_first, _rest) = loop {
            match self.text.split_first() {
                None => {
                    return Token {
                        ty: Type::End,
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
        let tok = Token {
            ty: Type::Symbol,
            pos: self.pos,
            text: &self.text[..1],
        };
        self.pos = match self.pos {
            Pos(i) => Pos(i + tok.text.len() as u32),
        };
        self.text = &self.text[1..];
        tok
    }
}
