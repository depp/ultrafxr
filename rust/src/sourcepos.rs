// A position within source text. The position represents a byte offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos(pub u32);

impl Pos {
    pub fn offset(&self) -> u32 {
        match self {
            Pos(offset) => *offset,
        }
    }
}

// A half-open range of source text. The positions represent byte offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

// Trait for objects which have locations in the source code.
pub trait HasPos {
    fn source_pos(&self) -> Span;
}

impl HasPos for Span {
    fn source_pos(&self) -> Span {
        *self
    }
}
