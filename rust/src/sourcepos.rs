use std::convert::TryFrom;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

// A position within source text. The position represents a byte offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos(pub u32);

// A half-open range of source text. The positions represent byte offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    /// An empty span that refers to no source locations.
    pub fn none() -> Self {
        Span {
            start: Pos(0),
            end: Pos(0),
        }
    }

    /// True if the span contains nothing.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the length of the span, in bytes.
    #[allow(dead_code)]
    pub fn len(&self) -> u32 {
        self.end.0 - self.end.0
    }

    /// Slice the span into smaller pieces, using the same index that you would
    /// use for the corresponding bytes.
    #[allow(dead_code)]
    pub fn sub_span(&self, range: impl SubSpan) -> Span {
        match range.get_sub_span(self) {
            Some(s) => s,
            None => panic!("span slice out of range"),
        }
    }
}

pub trait SubSpan {
    fn get_sub_span(&self, span: &Span) -> Option<Span>;
}

impl SubSpan for Range<usize> {
    fn get_sub_span(&self, span: &Span) -> Option<Span> {
        let offset = span.start.0;
        let start = Pos(offset.checked_add(u32::try_from(self.start).ok()?)?);
        let end = Pos(offset.checked_add(u32::try_from(self.end).ok()?)?);
        if start > end || end > span.end {
            return None;
        }
        Some(Span { start, end })
    }
}

impl SubSpan for RangeFrom<usize> {
    fn get_sub_span(&self, span: &Span) -> Option<Span> {
        let offset = span.start.0;
        let start = Pos(offset.checked_add(u32::try_from(self.start).ok()?)?);
        if start > span.end {
            return None;
        }
        Some(Span {
            start,
            end: span.end,
        })
    }
}

impl SubSpan for RangeFull {
    fn get_sub_span(&self, span: &Span) -> Option<Span> {
        Some(*span)
    }
}

impl SubSpan for RangeTo<usize> {
    fn get_sub_span(&self, span: &Span) -> Option<Span> {
        let offset = span.start.0;
        let end = Pos(offset.checked_add(u32::try_from(self.end).ok()?)?);
        if end > span.end {
            return None;
        }
        Some(Span {
            start: span.start,
            end,
        })
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn slice() {
        fn span(start: u32, end: u32) -> Span {
            Span {
                start: Pos(start),
                end: Pos(end),
            }
        }
        assert_eq!(span(5, 10).sub_span(1..2), span(6, 7));
        assert_eq!(span(5, 10).sub_span(1..), span(6, 10));
        assert_eq!(span(5, 10).sub_span(..2), span(5, 7));
        assert_eq!(span(5, 10).sub_span(..), span(5, 10));
    }
}
