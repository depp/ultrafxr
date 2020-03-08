use crate::color::{Style, StyleFlag};
use crate::error::{ErrorHandler, Severity};
use crate::sourcepos::Span;
use crate::sourceprint::write_source;
use crate::sourcetext::SourceText;
use std::fmt::Arguments;
use std::io;
use std::io::{stderr, Write};

const MESSAGE: Style<'static> = Style(&[StyleFlag::FgBrightWhite]);
const RESET: Style<'static> = Style(&[StyleFlag::Reset]);

/// Get the color style to use for a given severity level.
fn severity_color(severity: Severity) -> Style<'static> {
    use Severity::*;
    Style(match severity {
        Warning => &[StyleFlag::FgYellow, StyleFlag::Bold],
        Error => &[StyleFlag::FgRed, StyleFlag::Bold],
    })
}

// FIXME: Seems like we could combine these functions, but str is ?Sized.

/// Write a diagnostic message to a stream.
pub fn write_diagnostic(w: &mut impl Write, severity: Severity, msg: &str) -> io::Result<()> {
    writeln!(
        w,
        "{}{}{}: {}{}",
        severity_color(severity),
        severity,
        MESSAGE,
        msg,
        RESET
    )
}

/// Print a diagnostic message to stderr.
pub fn _print_diagnostic(severity: Severity, args: Arguments) {
    let stderr = stderr();
    let mut handle = stderr.lock();
    writeln!(
        &mut handle,
        "{}{}{}: {}{}",
        severity_color(severity),
        severity,
        MESSAGE,
        args,
        RESET,
    )
    .unwrap();
}

macro_rules! error {
    ($($arg:tt)*) => ({
        $crate::consolelogger::_print_diagnostic(
            $crate::error::Severity::Error,
            std::format_args!($($arg)*),
        )
    });
}

macro_rules! warning {
    ($($arg:tt)*) => ({
        $crate::consolelogger::_print_diagnostic(
            $crate::error::Severity::Warning,
            std::format_args!($($arg)*),
        )
    });
}

pub struct ConsoleLogger<'a> {
    text: Result<SourceText<'a>, (&'a str, &'a [u8])>,
}

impl<'a> ConsoleLogger<'a> {
    pub fn from_text(filename: &'a str, text: &'a [u8]) -> Self {
        return ConsoleLogger {
            text: Err((filename, text)),
        };
    }
    fn init(&mut self) {
        match self.text {
            Ok(_) => (),
            Err((filename, text)) => self.text = Ok(SourceText::new(filename, text)),
        }
    }
}

impl<'a> ErrorHandler for ConsoleLogger<'a> {
    fn handle(&mut self, pos: Span, message: &str) {
        self.init();
        let source_text = self.text.as_ref().unwrap();
        let mut stderr = stderr();
        write_diagnostic(&mut stderr, Severity::Error, message).unwrap();
        if let Some(text_pos) = source_text.span(pos) {
            write_source(&mut stderr, &source_text, &text_pos).unwrap();
        }
        writeln!(stderr).unwrap();
    }
}
