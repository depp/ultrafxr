use crate::color::{Style, StyleFlag};
use crate::error::Severity;
use std::fmt::Display;
use std::io;
use std::io::Write;

const MESSAGE: Style<'static> = Style(&[StyleFlag::FgBrightWhite]);
const RESET: Style<'static> = Style(&[StyleFlag::Reset]);

/// Get the color style to use for a given severity level.
fn severity_color(severity: Severity) -> Style<'static> {
    use Severity::*;
    Style(match severity {
        Error => &[StyleFlag::FgRed, StyleFlag::Bold],
    })
}

/// Write a diagnostic message to a stream.
pub fn write_diagnostic(
    w: &mut impl Write,
    severity: Severity,
    msg: &dyn Display,
) -> io::Result<()> {
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

// FIXME: Seems like we could combine these functions, but str is ?Sized.

/// Write a diagnostic message to a stream.
pub fn write_diagnostic_str(w: &mut impl Write, severity: Severity, msg: &str) -> io::Result<()> {
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
