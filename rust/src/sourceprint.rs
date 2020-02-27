use crate::sourcetext::*;
use crate::utf8::UTF8Segments;
use std::cmp::max;
use std::io;
use std::io::Write;

mod color {
    use crate::color::{Style, StyleFlag};
    use StyleFlag::*;
    pub const LINENO: Style<'static> = Style(&[FgBrightBlue]);
    pub const RESET: Style<'static> = Style(&[Reset]);
    pub const BADCHAR: Style<'static> = Style(&[ReverseVideo]);
    pub const HIGHLIGHT: Style<'static> = Style(&[FgBrightRed]);
}

const SPACES: [u8; 80] = [b' '; 80];
const CARETS: [u8; 80] = [b'^'; 80];

fn fill(w: &mut impl Write, count: usize, pattern: &[u8; 80]) -> io::Result<()> {
    let mut rem = count;
    while rem >= pattern.len() {
        w.write_all(&pattern[..])?;
        rem -= pattern.len();
    }
    if rem > 0 {
        w.write_all(&pattern[..rem])?;
    }
    Ok(())
}

struct SourcePrinter {
    column: usize,
    tab_width: u32,
    is_highlighted: bool,
}

impl SourcePrinter {
    fn write(&mut self, w: &mut impl Write, text: &[u8]) -> io::Result<()> {
        for seg in UTF8Segments(text) {
            match seg {
                Ok(s) => self.write_str(w, s)?,
                Err(s) => self.write_bytes(w, s)?,
            }
        }
        Ok(())
    }
    fn write_str(&mut self, w: &mut impl Write, s: &str) -> io::Result<()> {
        for c in s.chars() {
            match c {
                '\t' => {
                    self.stop_highlight(w)?;
                    let n = (self.tab_width - ((self.column as u32) % self.tab_width)) as usize;
                    fill(w, n, &SPACES)?;
                    self.column += n;
                }
                ' '..='\x7e' => {
                    self.stop_highlight(w)?;
                    write!(w, "{}", c as char)?;
                    self.column += 1;
                }
                _ => {
                    self.start_highlight(w)?;
                    let mut data: [u8; 10] = [0; 10];
                    let mut curs = io::Cursor::new(&mut data[..]);
                    write!(&mut curs, "<U+{:04X}>", c as u32).unwrap();
                    let pos = curs.position() as usize;
                    w.write_all(&data[..pos])?;
                    self.column += pos;
                }
            }
        }
        Ok(())
    }
    fn write_bytes(&mut self, w: &mut impl Write, s: &[u8]) -> io::Result<()> {
        for &c in s.iter() {
            write!(w, "<{:02X}>", c)?;
            self.column += 4;
        }
        Ok(())
    }
    fn finish(mut self, w: &mut impl Write) -> io::Result<()> {
        self.stop_highlight(w)
    }
    fn start_highlight(&mut self, w: &mut impl Write) -> io::Result<()> {
        if !self.is_highlighted {
            self.is_highlighted = true;
            write!(w, "{}", color::BADCHAR)
        } else {
            Ok(())
        }
    }
    fn stop_highlight(&mut self, w: &mut impl Write) -> io::Result<()> {
        if self.is_highlighted {
            self.is_highlighted = false;
            write!(w, "{}", color::RESET)
        } else {
            Ok(())
        }
    }
}

/// Return the number of digits in the given number.
fn digit_length(n: usize) -> usize {
    let mut data = [0; 20];
    let mut curs = io::Cursor::new(&mut data[..]);
    write!(&mut curs, "{}", n).unwrap();
    curs.position() as usize
}

/// Write a section of source code with the given range highlighted.
///
/// Control characters, non-ASCII characters, and invalid UTF-8 sequences are
/// appropriately formatted and made visible.
pub fn write_source(w: &mut impl Write, text: &SourceText<'_>, span: &TextSpan) -> io::Result<()> {
    let lineno_len = digit_length((max(span.start.line, span.end.line) + 1) as usize);
    fill(w, lineno_len, &SPACES)?;
    writeln!(
        w,
        "{}-->{} {}:{}:{}",
        color::LINENO,
        color::RESET,
        text.filename(),
        span.start.line + 1,
        span.start.byte
    )?;
    for lineno in span.start.line..=span.end.line {
        fill(w, lineno_len + 1, &SPACES)?;
        writeln!(w, "{}|{}", color::LINENO, color::RESET)?;
        write!(w, "{}{} |{} ", color::LINENO, lineno + 1, color::RESET)?;
        let line = text.line(lineno);
        let startbyte = if lineno == span.start.line {
            span.start.byte as usize
        } else {
            0
        };
        let endbyte = if lineno == span.end.line {
            span.end.byte as usize
        } else {
            line.len()
        };
        let mut pr = SourcePrinter {
            column: 0,
            tab_width: 8,
            is_highlighted: false,
        };
        pr.write(w, &line[..startbyte])?;
        let startcol = pr.column;
        pr.write(w, &line[startbyte..endbyte])?;
        let mut endcol = pr.column;
        pr.write(w, &line[endbyte..])?;
        if span.end.line == lineno && startcol == endcol {
            endcol = startcol + 1;
        }
        pr.finish(w)?;
        writeln!(w)?;
        if startcol < endcol {
            fill(w, lineno_len + 1, &SPACES)?;
            write!(w, "{}|{} ", color::LINENO, color::RESET)?;
            fill(w, startcol, &SPACES)?;
            write!(w, "{}", color::HIGHLIGHT)?;
            fill(w, endcol - startcol, &CARETS)?;
            write!(w, "{}\n", color::RESET)?;
        }
    }
    Ok(())
}
