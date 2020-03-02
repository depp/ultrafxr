use crate::error::ErrorHandler;
use crate::sexpr::{Content, SExpr};
use crate::sourcepos::{HasPos, Span};
use crate::token::{Token, Tokenizer, Type};
use std::fmt::Write;
use std::str::from_utf8;
use std::vec::Vec;

pub struct Parser {
    exprs: Vec<SExpr>,
    groups: Vec<(Span, usize)>,
}

// Convert a token to an owned string.
fn tok_str(tok: &Token) -> Box<str> {
    match from_utf8(tok.text) {
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

// Send an error message
fn handle_error_token(err_handler: &mut dyn ErrorHandler, pos: Span, text: &[u8]) {
    let msg: String = match from_utf8(text) {
        Ok(s) => match s.chars().next() {
            Some(c) => {
                if c <= '\x1f' || ('\u{7f}' <= c && c <= '\u{9f}') {
                    format!("unexpected control character U+{:04X}", c as u32)
                } else if c <= '\u{7f}' {
                    if c == '\'' {
                        "unexpected character <'>".to_owned()
                    } else {
                        format!("unexpected character '{}'", c)
                    }
                } else {
                    format!("unexpected Unicode character U+{:04X}", c as u32)
                }
            }
            // Tokenizer should not produce this.
            _ => panic!("empty error token"),
        },
        Err(_) => {
            if text.is_empty() {
                // Tokenizer should not produce this.
                panic!("empty error token");
            }
            let mut s = String::new();
            for b in text.iter() {
                write!(&mut s, "0x{:02x}, ", b).unwrap();
            }
            format!("invalid UTF-8 text (byte sequence {})", &s[..s.len() - 2])
        }
    };
    err_handler.handle(pos, msg.as_ref());
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
                    handle_error_token(err_handler, pos, tok.text);
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
