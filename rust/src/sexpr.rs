use crate::error::ErrorHandler;
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

pub struct Parser {
    exprs: Vec<SExpr>,
    groups: Vec<(Span, usize)>,
}

// Convert a token to an owned string.
fn tok_str(tok: &Token) -> Box<str> {
    match std::str::from_utf8(tok.text) {
        Ok(s) => Box::from(s),
        // The tokenizer is supposed to check that the non-error tokens are
        // valid UTF-8, but this is not guaranteed by the type system.
        Err(_) => panic!("invalid token from tokenizer"),
    }
}

// A result from running the parser.
pub enum ParseResult {
    None,         // Token stream ended without any expressions in it.
    Incomplete,   // Token stream ended in middle of expression.
    Error,        // Did not reach end of token stream, encountered error.
    Value(SExpr), // Parsed complete expression.
}

impl Parser {
    pub fn new() -> Self {
        return Parser {
            exprs: Vec::new(),
            groups: Vec::new(),
        };
    }

    // Parse the next s-expression from the token stream.
    pub fn parse(
        &mut self,
        err_handler: &mut dyn ErrorHandler,
        tokenizer: &mut Tokenizer,
    ) -> ParseResult {
        loop {
            let tok = tokenizer.next();
            let pos = tok.source_pos();
            match tok.ty {
                Type::End => {
                    return if self.groups.is_empty() {
                        ParseResult::None
                    } else {
                        ParseResult::Incomplete
                    }
                }
                Type::Error => {
                    err_handler.handle(pos, "unexpected character");
                    return ParseResult::Error;
                }
                Type::Comment => {}
                Type::Symbol => {
                    let expr = SExpr {
                        pos,
                        content: Content::Symbol(tok_str(&tok)),
                    };
                    if self.groups.is_empty() {
                        return ParseResult::Value(expr);
                    }
                    self.exprs.push(expr);
                }
                Type::Number => {
                    let expr = SExpr {
                        pos,
                        content: Content::Number(tok_str(&tok)),
                    };
                    if self.groups.is_empty() {
                        return ParseResult::Value(expr);
                    }
                    self.exprs.push(expr);
                }
                Type::ParenOpen => {
                    self.groups.push((pos, self.exprs.len()));
                }
                Type::ParenClose => match self.groups.pop() {
                    Some((start_pos, offset)) => {
                        let items: Box<[SExpr]> = self.exprs.drain(offset..).collect();
                        let expr = SExpr {
                            pos: Span {
                                start: start_pos.start,
                                end: pos.source_pos().end,
                            },
                            content: Content::List(items),
                        };
                        if self.groups.is_empty() {
                            return ParseResult::Value(expr);
                        }
                        self.exprs.push(expr);
                    }
                    _ => {
                        err_handler.handle(pos, "extra ')'");
                        return ParseResult::Error;
                    }
                },
            }
        }
    }

    // Finish parsing a document, and report errors for any unclosed groups.
    pub fn finish(&self, err_handler: &mut dyn ErrorHandler) {
        for (pos, _) in self.groups.iter().rev() {
            err_handler.handle(*pos, "missing ')'");
            return;
        }
    }
}
