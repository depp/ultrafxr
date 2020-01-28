use crate::sourcepos::{HasPos, Span};
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

pub trait ErrorHandler {
    fn handle(&mut self, pos: Span, message: &str);
}

impl SExpr {
    // Parse a sequence of s-expressions from a token stream.
    pub fn parse(
        err_handler: &mut dyn ErrorHandler,
        tokenizer: &mut Tokenizer,
    ) -> Option<Box<[Self]>> {
        fn tok_str(tok: &Token) -> Box<str> {
            match std::str::from_utf8(tok.text) {
                Ok(s) => Box::from(s),
                // The tokenizer is supposed to check that the non-error tokens
                // are valid UTF-8, but this is not guaranteed by the type
                // system.
                Err(_) => panic!("invalid token from tokenizer"),
            }
        }
        let mut exprs = Vec::<SExpr>::new();
        let mut groups = Vec::<(Span, usize)>::new();
        loop {
            let tok = tokenizer.next();
            let pos = tok.source_pos();
            match tok.ty {
                Type::End => {
                    return match groups.pop() {
                        Some((pos, _)) => {
                            err_handler.handle(pos, "unmatched '('");
                            None
                        }
                        None => Some(exprs.into_boxed_slice()),
                    }
                }
                Type::Error => {
                    err_handler.handle(pos, "unexpected character");
                    return None;
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
                    groups.push((pos, exprs.len()));
                }
                Type::ParenClose => match groups.pop() {
                    Some((start_pos, offset)) => {
                        let items: Box<[SExpr]> = exprs.drain(offset..).collect();
                        exprs.push(SExpr {
                            pos: Span {
                                start: start_pos.start,
                                end: pos.source_pos().end,
                            },
                            content: Content::List(items),
                        });
                    }
                    _ => {
                        err_handler.handle(pos, "extra ')'");
                        return None;
                    }
                },
            }
        }
    }
}
