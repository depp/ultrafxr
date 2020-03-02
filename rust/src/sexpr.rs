use crate::sourcepos::{HasPos, Span};

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
