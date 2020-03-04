use crate::sourcepos::{HasPos, Span};
use crate::units::Units;
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

/// The type of an s-expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Symbol,
    Integer,
    Float,
    List,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use Type::*;
        f.write_str(match self {
            Symbol => "symbol",
            Integer => "integer",
            Float => "float",
            List => "list",
        })
    }
}

/// The contents of an s-expression.
#[derive(Debug)]
pub enum Content {
    Symbol(Box<str>),
    Integer(Units, i64),
    Float(Units, f64),
    List(Box<[SExpr]>),
}

impl Content {
    /// Get the content's type.
    fn get_type(&self) -> Type {
        match self {
            Content::Symbol(_) => Type::Symbol,
            Content::Integer(_, _) => Type::Integer,
            Content::Float(_, _) => Type::Float,
            Content::List(_) => Type::List,
        }
    }
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
    /// Get the expression's type.
    pub fn get_type(&self) -> Type {
        self.content.get_type()
    }

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
