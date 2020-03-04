use crate::sourcepos::{HasPos, Span};
use crate::units::Units;
use std::fmt::Write;

/// The contents of an s-expression.
#[derive(Debug)]
pub enum Content {
    Symbol(Box<str>),
    Integer(Units, i64),
    Float(Units, f64),
    List(Box<[SExpr]>),
}

/// An s-expression.
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
    /// Print the s-expression to a string.
    pub fn print(&self) -> String {
        let mut out = String::new();
        self.print_impl(&mut out);
        out
    }

    fn print_impl(&self, out: &mut String) {
        use Content::*;
        match &self.content {
            Symbol(sym) => out.push_str(sym),
            Integer(units, num) => {
                if units.is_scalar() {
                    write!(out, "{}", num).unwrap();
                } else {
                    write!(out, "[{} {}]", units, num).unwrap();
                }
            }
            Float(units, num) => {
                if units.is_scalar() {
                    write!(out, "{}", num).unwrap();
                } else {
                    write!(out, "[{} {}]", units, num).unwrap();
                }
            }
            List(list) => {
                out.push('(');
                let mut iter = list.iter();
                match iter.next() {
                    None => (),
                    Some(item) => {
                        item.print_impl(out);
                        for item in iter {
                            out.push(' ');
                            item.print_impl(out);
                        }
                    }
                }
                out.push(')');
            }
        }
    }
}
