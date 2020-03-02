use crate::sourcepos::{HasPos, Span};

/// The contents of an s-expression.
#[derive(Debug)]
pub enum Content {
    Symbol(Box<str>),
    Number(Box<str>),
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
