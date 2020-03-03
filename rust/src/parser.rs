use crate::error::ErrorHandler;
use crate::number::ParsedNumber;
use crate::sexpr::{Content, SExpr};
use crate::sourcepos::{HasPos, Span};
use crate::token::{Token, Tokenizer, Type};
use crate::units::Units;
use std::fmt::Write;
use std::str;

/// An incremental s-expression parser.
pub struct Parser {
    exprs: Vec<SExpr>,
    groups: Vec<(Span, usize)>,
    number: ParsedNumber,
}

/// Get the contents of a token as a string.
///
/// This will panic if called on invalid UTF-8, which should only be true for
/// some Error tokens.
fn tok_str<'a>(tok: &Token<'a>) -> &'a str {
    match str::from_utf8(tok.text) {
        Ok(s) => s,
        Err(_) => panic!("invalid token from tokenizer"),
    }
}

// Convert a token to an owned string.
fn tok_boxstr(tok: &Token) -> Box<str> {
    match str::from_utf8(tok.text) {
        Ok(s) => Box::from(s),
        // The tokenizer is supposed to check that the non-error tokens are
        // valid UTF-8, but this is not guaranteed by the type system.
        Err(_) => panic!("invalid token from tokenizer"),
    }
}

/// A result from running the parser.
pub enum ParseResult {
    None,         // Token stream ended without any expressions in it.
    Incomplete,   // Token stream ended in middle of expression.
    Error,        // Did not reach end of token stream, encountered error.
    Value(SExpr), // Parsed complete expression.
}

// Send an error message
fn handle_error_token(err_handler: &mut dyn ErrorHandler, pos: Span, text: &[u8]) {
    let msg: String = match str::from_utf8(text) {
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
            number: ParsedNumber::new(),
        };
    }

    /// Parse the next s-expression from the token stream.
    ///
    /// If the stream ends without producing a complete s-expression, parse()
    /// can be called again to continue parsing incrementally.
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
                        content: Content::Symbol(tok_boxstr(&tok)),
                    };
                    if self.groups.is_empty() {
                        return ParseResult::Value(expr);
                    }
                    self.exprs.push(expr);
                }
                Type::Number => {
                    let content = match self.parse_number(err_handler, &tok) {
                        Some(x) => x,
                        None => return ParseResult::Error,
                    };
                    let expr = SExpr { pos, content };
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

    /// Finish parsing a document, and report errors for any unclosed groups.
    pub fn finish(&self, err_handler: &mut dyn ErrorHandler) {
        for (pos, _) in self.groups.iter().rev() {
            err_handler.handle(*pos, "missing ')'");
            return;
        }
    }

    /// Parse a numeric token.
    fn parse_number(&mut self, err_handler: &mut dyn ErrorHandler, tok: &Token) -> Option<Content> {
        let tokpos = tok.source_pos();
        let text = tok_str(tok);
        let rest = match self.number.parse(text, tokpos) {
            Ok(rest) => rest,
            Err((e, pos)) => {
                err_handler.handle(pos, e.to_string().as_ref());
                return None;
            }
        };
        let idx = text.len() - rest.len();
        let (_upos, units, exponent) = match Units::parse(rest, tokpos.sub_span(idx..)) {
            Ok(r) => r,
            Err((e, pos)) => {
                err_handler.handle(pos, e.to_string().as_ref());
                return None;
            }
        };
        if exponent != 0 {
            self.number.exponent = Some(self.number.exponent.unwrap_or(0).saturating_add(exponent));
        }
        self.number.trim();
        let num = Box::from(self.number.to_string());
        Some(if units == Default::default() {
            Content::Number(num)
        } else {
            Content::Units(units, num)
        })
    }
}
