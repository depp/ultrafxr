use std::fmt;

/// A Style is a collection of terminal style settings.
#[derive(Debug, Clone, Copy)]
pub struct Style<'a>(pub &'a [StyleFlag]);

/// A StyleFlag is an individual terminal style setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum StyleFlag {
    Reset = 0,
    Bold = 1,
    FgBlack = 30,
    FgRed = 31,
    FgGreen = 32,
    FgYellow = 33,
    FgBlue = 34,
    FgMagenta = 35,
    FgCyan = 36,
    FgWhite = 37,
}

impl fmt::Display for Style<'_> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        f.write_str("\x1b[")?;
        write!(&mut f, "{}", self.0[0] as u8)?;
        for &flag in self.0[1..].iter() {
            write!(&mut f, ";{}", flag as u8)?;
        }
        f.write_str("m")
    }
}
