use crate::sourcepos::{HasPos, Pos, Span};
use crate::token::{Token, Tokenizer, Type};
use std::vec::Vec;

#[derive(Debug)]
pub enum Content {
    Symbol(Box<str>),
    Number(Box<str>),
    List(Box<[SExpr]>),
}

#[derive(Debug)]
pub struct SExpr {
    pub pos: Span,
    pub content: Content,
}

impl HasPos for SExpr {
    fn source_pos(&self) -> Span {
        self.pos
    }
}

impl SExpr {
    // Parse a sequence of s-expressions from a token stream.
    pub fn parse(tokenizer: &mut Tokenizer) -> Box<[Self]> {
        fn tok_str(tok: &Token) -> Box<str> {
            // Can panic.
            Box::from(std::str::from_utf8(tok.text).unwrap())
        }
        let mut exprs = Vec::<SExpr>::new();
        let mut groups = Vec::<(Pos, usize)>::new();
        loop {
            let tok = tokenizer.next();
            let pos = tok.source_pos();
            match tok.ty {
                Type::End => {
                    if !groups.is_empty() {
                        panic!("missing ')'");
                    }
                    break;
                }
                Type::Error => {
                    panic!("error token");
                }
                Type::Comment => {}
                Type::Symbol => exprs.push(SExpr {
                    pos,
                    content: Content::Symbol(tok_str(&tok)),
                }),
                Type::Number => exprs.push(SExpr {
                    pos,
                    content: Content::Number(tok_str(&tok)),
                }),
                Type::ParenOpen => {
                    groups.push((pos.start, exprs.len()));
                }
                Type::ParenClose => match groups.pop() {
                    Some((start, offset)) => {
                        let items: Box<[SExpr]> = exprs.drain(offset..).collect();
                        exprs.push(SExpr {
                            pos: Span {
                                start,
                                end: pos.source_pos().end,
                            },
                            content: Content::List(items),
                        });
                    }
                    _ => {
                        panic!("extra ')'");
                    }
                },
            }
        }
        exprs.into_boxed_slice()
    }
}
