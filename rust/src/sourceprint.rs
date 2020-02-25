use crate::sourcetext::*;
use crate::utf8::UTF8Segments;
use std::cmp::max;
use std::fmt::Write;
use std::io;

mod color {
    use crate::color::{Style, StyleFlag};
    use StyleFlag::*;
    pub const LINENO: Style<'static> = Style(&[FgBrightBlue]);
    pub const RESET: Style<'static> = Style(&[Reset]);
    pub const BADCHAR: Style<'static> = Style(&[ReverseVideo]);
    pub const HIGHLIGHT: Style<'static> = Style(&[FgBrightRed]);
}

struct SourcePrinter {
    buf: String,
    column: u32,
    tab_width: u32,
    is_highlighted: bool,
}

impl SourcePrinter {
    fn write(&mut self, text: &[u8]) {
        for seg in UTF8Segments(text) {
            match seg {
                Ok(s) => self.write_str(s),
                Err(s) => self.write_bytes(s),
            }
        }
    }
    fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\t' => {
                    self.stop_highlight();
                    let n = self.tab_width - (self.column % self.tab_width);
                    for _ in 0..n {
                        self.buf.push(' ');
                    }
                    self.column += n;
                }
                ' '..='\x7e' => {
                    self.stop_highlight();
                    self.buf.push(c);
                    self.column += 1;
                }
                _ => {
                    self.start_highlight();
                    let start = self.buf.len() as u32;
                    write!(&mut self.buf, "<U+{:04X}>", c as u32).unwrap();
                    self.column += (self.buf.len() as u32) - start;
                }
            }
        }
    }
    fn write_bytes(&mut self, s: &[u8]) {
        for &c in s.iter() {
            write!(&mut self.buf, "<{:02X}>", c).unwrap();
            self.column += 4;
        }
    }
    fn finish(mut self) -> String {
        self.stop_highlight();
        self.buf
    }
    fn start_highlight(&mut self) {
        if !self.is_highlighted {
            write!(&mut self.buf, "{}", color::BADCHAR).unwrap();
            self.is_highlighted = true;
        }
    }
    fn stop_highlight(&mut self) {
        if self.is_highlighted {
            write!(&mut self.buf, "{}", color::RESET).unwrap();
            self.is_highlighted = false;
        }
    }
}

// Print a section of source code with the given range highlighted.
pub fn print_source(
    output: &mut dyn io::Write,
    text: &SourceText<'_>,
    span: &TextSpan,
) -> io::Result<()> {
    let mut buf = String::new();
    write!(&mut buf, "{}", max(span.start.line, span.end.line) + 1).unwrap();
    let lineno_len = buf.len();
    buf.clear();
    for lineno in span.start.line..=span.end.line {
        write!(
            &mut buf,
            "{}{} |{} ",
            color::LINENO,
            lineno + 1,
            color::RESET
        )
        .unwrap();
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
            buf,
            column: 0,
            tab_width: 8,
            is_highlighted: false,
        };
        pr.write(&line[..startbyte]);
        let startcol = pr.column;
        pr.write(&line[startbyte..endbyte]);
        let mut endcol = pr.column;
        pr.write(&line[endbyte..]);
        if span.end.line == lineno && startcol == endcol {
            endcol = startcol + 1;
        }
        buf = pr.finish();
        buf.push('\n');
        if startcol < endcol {
            for _ in 0..(lineno_len + 1) {
                buf.push(' ');
            }
            write!(&mut buf, "{}|{} ", color::LINENO, color::RESET).unwrap();
            for _ in 0..startcol {
                buf.push(' ');
            }
            write!(&mut buf, "{}", color::HIGHLIGHT).unwrap();
            for _ in 0..(endcol - startcol) {
                buf.push('^');
            }
            write!(&mut buf, "{}\n", color::RESET).unwrap();
        }
    }
    output.write(buf.as_ref())?;
    Ok(())
}
