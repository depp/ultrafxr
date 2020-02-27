use crate::sourcepos::Span;
use std::fmt;

// An object that handles errors during parsing or evaluation.
pub trait ErrorHandler {
    fn handle(&mut self, pos: Span, message: &str);
}

/// Serevrity level for diagnostic messages.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum Severity {
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Severity::*;
        f.write_str(match *self {
            Error => "error",
        })
    }
}
