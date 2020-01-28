use crate::sourcepos::Span;

// An object that handles errors during parsing or evaluation.
pub trait ErrorHandler {
    fn handle(&mut self, pos: Span, message: &str);
}
