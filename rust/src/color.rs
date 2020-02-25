use std::fmt;

/// A Style is a collection of terminal style settings.
#[derive(Debug, Clone, Copy)]
pub struct Style<'a>(pub &'a [StyleFlag]);

/// A StyleFlag is an individual terminal style setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum StyleFlag {
    // Reset all styles.
    Reset = 0,

    // General styles.
    Bold = 1,
    Faint = 2,
    Underline = 4,
    Blink = 5,
    ReverseVideo = 7,
    NormalIntensity = 22,
    UnderlineOff = 24,
    BlinkOff = 25,
    ReverseVideoOff = 27,

    // Foreground colors.
    FgBlack = 30,
    FgRed = 31,
    FgGreen = 32,
    FgYellow = 33,
    FgBlue = 34,
    FgMagenta = 35,
    FgCyan = 36,
    FgWhite = 37,
    FgBrightBlack = 90,
    FgBrightRed = 91,
    FgBrightGreen = 92,
    FgBrightYellow = 93,
    FgBrightBlue = 94,
    FgBrightMagenta = 95,
    FgBrightCyan = 96,
    FgBrightWhite = 97,

    // Background colors.
    BgBlack = 40,
    BgRed = 41,
    BgGreen = 42,
    BgYellow = 43,
    BgBlue = 44,
    BgMagenta = 45,
    BgCyan = 46,
    BgWhite = 47,
    BgBrightBlack = 100,
    BgBrightRed = 101,
    BgBrightGreen = 102,
    BgBrightYellow = 103,
    BgBrightBlue = 104,
    BgBrightMagenta = 105,
    BgBrightCyan = 106,
    BgBrightWhite = 107,
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
