use crate::sourcepos::{Pos, Span};
use std::fmt;

/// A type of error from parsing a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidDigit(Radix, char),
    ExtraPoint,
    UnexpectedPoint(Radix),
    UnexpectedChar(char),
    NoDigits,
    NoExponentValue,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseError::*;
        match *self {
            InvalidDigit(radix, c) => write!(f, "invalid digit for base {}: {:?}", radix as u8, c),
            ExtraPoint => write!(f, "unexpected extra '.'"),
            UnexpectedPoint(radix) => {
                write!(f, "non-integers not supported in base {}", radix as u8)
            }
            UnexpectedChar(c) => write!(f, "unexpected character {:?}", c),
            NoDigits => write!(f, "number has no digits"),
            NoExponentValue => write!(f, "missing exponent value"),
        }
    }
}

/// The sign for a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
}

/// A numeric radix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Radix {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}

/// A number which has been parsed into its parts.
///
/// Digits are stored least-significant first.
#[derive(Debug, Clone)]
pub struct ParsedNumber {
    pub sign: Sign,
    pub radix: Radix,
    pub digits: Vec<u8>,
    pub exponent: Option<i32>,
}

fn is_digit(c: char) -> bool {
    '0' <= c && c <= '9'
}

fn is_hex_digit(c: char) -> bool {
    '0' <= c && c <= '9' || 'a' <= c && c <= 'f' || 'A' <= c && c <= 'F'
}

fn parse_digit(c: char) -> u8 {
    match c {
        '0'..='9' => (c as u32 - '0' as u32) as u8,
        'a'..='z' => (c as u32 - 'a' as u32) as u8 + 10,
        'A'..='Z' => (c as u32 - 'A' as u32) as u8 + 10,
        _ => u8::max_value(),
    }
}

fn starts_with_digit(s: &str) -> bool {
    match s.chars().next() {
        Some(c) if is_digit(c) => true,
        _ => false,
    }
}

fn starts_with_hex_digit(s: &str) -> bool {
    match s.chars().next() {
        Some(c) if is_hex_digit(c) => true,
        _ => false,
    }
}

/// Parse an exponent from a string.
///
/// Return the exponent's value, clamped to the range of i32, and the remainder
/// of the string after the exponent.
fn parse_exponent(text: &str, pos: Span) -> Result<(Option<i32>, &str), (ParseError, Span)> {
    let mut chars = text.chars();
    let mut value: u32 = 0;
    let mut has_value = false;
    let sign = match chars.next() {
        Some(c) if c == 'e' || c == 'E' => match chars.next() {
            Some(c) => match c {
                '+' => Sign::Positive,
                '-' => Sign::Negative,
                '0'..='9' => {
                    value = c as u32 - '0' as u32;
                    has_value = true;
                    Sign::Positive
                }
                _ => return Ok((None, text)),
            },
            _ => return Ok((None, text)),
        },
        _ => return Ok((None, text)),
    };
    let rest = loop {
        let rest = chars.as_str();
        match chars.next() {
            Some(c) if is_digit(c) => {
                value = value.saturating_mul(10);
                value = value.saturating_add(c as u32 - '0' as u32);
                has_value = true;
            }
            _ => break rest,
        }
    };
    if !has_value {
        return Err((
            ParseError::NoExponentValue,
            pos.sub_span(..text.len() - rest.len()),
        ));
    }
    let value = match sign {
        Sign::Positive => {
            if value > i32::max_value() as u32 {
                i32::max_value()
            } else {
                value as i32
            }
        }
        Sign::Negative => {
            if value > i32::max_value() as u32 {
                i32::min_value()
            } else {
                -(value as i32)
            }
        }
    };
    Ok((Some(value), rest))
}

impl ParsedNumber {
    /// Create an empty parsed number.
    pub fn new() -> Self {
        return ParsedNumber {
            sign: Sign::Positive,
            radix: Radix::Decimal,
            digits: Vec::new(),
            exponent: None,
        };
    }

    /// Parse a number from its textual representation.
    ///
    /// Returns the remainder of the string, which appears after the number.
    pub fn parse<'a>(&mut self, text: &'a str, pos: Span) -> Result<&'a str, (ParseError, Span)> {
        let toklen = text.len();
        let mut chars = text.chars();
        let (sign, text) = match chars.next() {
            Some('+') => (Sign::Positive, chars.as_str()),
            Some('-') => (Sign::Negative, chars.as_str()),
            _ => (Sign::Positive, text),
        };
        let pos = pos.sub_span(toklen - text.len()..);
        self.sign = sign;
        self.digits.clear();
        self.exponent = None;
        let mut chars = text.chars();
        if chars.next() == Some('0') {
            match chars.next() {
                Some(c) => {
                    let text = chars.as_str();
                    match c {
                        'b' | 'B' if starts_with_digit(text) => {
                            return self.parse_int(Radix::Binary, text, pos);
                        }
                        'o' | 'O' if starts_with_digit(text) => {
                            return self.parse_int(Radix::Octal, text, pos);
                        }
                        'x' | 'X' if starts_with_hex_digit(text) => {
                            return self.parse_int(Radix::Hexadecimal, text, pos);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        self.parse_dec(text, pos)
    }

    /// Parse an integer, without sign, and return the remainder of the string.
    fn parse_int<'a>(
        &mut self,
        radix: Radix,
        text: &'a str,
        pos: Span,
    ) -> Result<&'a str, (ParseError, Span)> {
        self.radix = radix;
        let mut chars = text.chars();
        loop {
            let rest = chars.as_str();
            match chars.next() {
                Some(c) => {
                    let d = parse_digit(c);
                    if d >= radix as u8 {
                        return Err((
                            if d < 10 {
                                ParseError::InvalidDigit(radix, c)
                            } else if c == '.' {
                                ParseError::UnexpectedPoint(radix)
                            } else {
                                ParseError::UnexpectedChar(c)
                            },
                            pos.sub_span(
                                text.len() - rest.len()..text.len() - chars.as_str().len(),
                            ),
                        ));
                    }
                    self.digits.push(d);
                }
                _ => {
                    self.digits.reverse();
                    return Ok(rest);
                }
            }
        }
    }

    /// Parse a decimal number, without sign, and return the remainder of the string.
    fn parse_dec<'a>(&mut self, text: &'a str, pos: Span) -> Result<&'a str, (ParseError, Span)> {
        let toklen = text.len();
        self.radix = Radix::Decimal;
        let (frac_digits, text) = self.parse_mantissa(toklen, text)?;
        if self.digits.is_empty() {
            return Err((ParseError::NoDigits, pos));
        }
        let pos = pos.sub_span(toklen - text.len()..);
        self.digits.reverse();
        let (exponent, text) = parse_exponent(text, pos)?;
        self.exponent = match frac_digits {
            Some(count) => Some({
                let bias = if count > i32::max_value() as usize {
                    i32::min_value()
                } else {
                    -(count as i32)
                };
                match exponent {
                    Some(value) => {
                        if value == i32::min_value() || value == i32::max_value() {
                            value
                        } else {
                            value.saturating_add(bias)
                        }
                    }
                    None => bias,
                }
            }),
            _ => exponent,
        };
        Ok(text)
    }

    /// Parse the mantissa of a decimal number. Return the number of digits past
    /// the decimal point and the remainder of the string.
    ///
    /// Pushes the most significant digit first.
    fn parse_mantissa<'a>(
        &mut self,
        toklen: usize,
        text: &'a str,
    ) -> Result<(Option<usize>, &'a str), (ParseError, Span)> {
        let mut chars = text.chars();
        let point_pos = loop {
            let rest = chars.as_str();
            match chars.next() {
                Some(c) => match c {
                    '0'..='9' => self.digits.push((c as u32 - '0' as u32) as u8),
                    '.' => break self.digits.len(),
                    _ => return Ok((None, rest)),
                },
                _ => return Ok((None, rest)),
            }
        };
        let rest = loop {
            let rest = chars.as_str();
            match chars.next() {
                Some(c) => match c {
                    '0'..='9' => self.digits.push((c as u32 - '0' as u32) as u8),
                    '.' => {
                        return Err((
                            ParseError::ExtraPoint,
                            Span {
                                start: Pos((toklen - rest.len()) as u32),
                                end: Pos((toklen - chars.as_str().len()) as u32),
                            },
                        ));
                    }
                    _ => break rest,
                },
                _ => break rest,
            }
        };
        Ok((Some(self.digits.len() - point_pos), rest))
    }

    /// Trim a number by removing leading and trailing zeroes where possible.
    /// This may remove all digits from the number, if they are all zero.
    pub fn trim(&mut self) {
        fn nonzero(c: &u8) -> bool {
            *c != 0
        }
        self.digits
            .truncate(match self.digits.iter().rev().position(nonzero) {
                Some(n) => self.digits.len() - n,
                None => 0,
            });
        if let Some(exponent) = self.exponent {
            let n = match self.digits.iter().position(nonzero) {
                Some(n) => n,
                None => self.digits.len(),
            };
            self.digits.drain(..n);
            self.exponent = Some(exponent.saturating_add(n as i32));
        }
    }
}

impl ToString for ParsedNumber {
    fn to_string(&self) -> String {
        use std::fmt::Write;
        let mut length = if self.digits.is_empty() {
            1
        } else {
            self.digits.len()
        };
        if self.sign == Sign::Negative {
            length += 1;
        }
        if self.radix == Radix::Decimal {
            if self.exponent.is_some() {
                length += 13;
            }
        } else {
            length += 2;
        }
        let mut s = String::with_capacity(length);
        if self.sign == Sign::Negative {
            s.push('-');
        }
        s.push_str(match self.radix {
            Radix::Binary => "0b",
            Radix::Octal => "0o",
            Radix::Decimal => "",
            Radix::Hexadecimal => "0x",
        });
        const DIGITS: [u8; 16] = *b"0123456789abcdef";
        if self.digits.is_empty() {
            s.push('0');
        } else {
            for &d in self.digits.iter().rev() {
                s.push(DIGITS[d as usize] as char);
            }
        }
        match self.exponent {
            Some(exp) => write!(&mut s, "e{:+}", exp).unwrap(),
            None => (),
        }
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_success() {
        let mut success = true;
        type Case = (
            &'static str,
            &'static str,
            Sign,
            Radix,
            &'static [u8],
            Option<i32>,
        );
        use Radix::*;
        use Sign::*;
        const CASES: &'static [Case] = &[
            ("0", "", Positive, Decimal, &[0], None),
            (
                "0x0123456789abcdef",
                "",
                Positive,
                Hexadecimal,
                &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
                None,
            ),
            (
                "0XABCDEF",
                "",
                Positive,
                Hexadecimal,
                &[15, 14, 13, 12, 11, 10],
                None,
            ),
            ("0b011", "", Positive, Binary, &[1, 1, 0], None),
            ("0B110", "", Positive, Binary, &[0, 1, 1], None),
            (
                "0o01234567",
                "",
                Positive,
                Octal,
                &[7, 6, 5, 4, 3, 2, 1, 0],
                None,
            ),
            (
                "0O76543210",
                "",
                Positive,
                Octal,
                &[0, 1, 2, 3, 4, 5, 6, 7],
                None,
            ),
            ("1.", "", Positive, Decimal, &[1], Some(0)),
            (".9", "", Positive, Decimal, &[9], Some(-1)),
            ("1e+15", "", Positive, Decimal, &[1], Some(15)),
            ("2e10", "", Positive, Decimal, &[2], Some(10)),
            ("9e-99", "", Positive, Decimal, &[9], Some(-99)),
            ("1.2e3", "", Positive, Decimal, &[2, 1], Some(2)),
            ("5.6e-3", "", Positive, Decimal, &[6, 5], Some(-4)),
            ("-0", "", Negative, Decimal, &[0], None),
            ("-0b11", "", Negative, Binary, &[1, 1], None),
            ("-0x123", "", Negative, Hexadecimal, &[3, 2, 1], None),
            ("-0o777", "", Negative, Octal, &[7, 7, 7], None),
            ("+123", "", Positive, Decimal, &[3, 2, 1], None),
            ("+0x123", "", Positive, Hexadecimal, &[3, 2, 1], None),
            ("12V", "V", Positive, Decimal, &[2, 1], None),
            ("0o", "o", Positive, Decimal, &[0], None),
            ("0oct", "oct", Positive, Decimal, &[0], None),
            ("0x", "x", Positive, Decimal, &[0], None),
            ("0xyz", "xyz", Positive, Decimal, &[0], None),
            ("0b", "b", Positive, Decimal, &[0], None),
            ("0bbb", "bbb", Positive, Decimal, &[0], None),
            ("1.2e3ms", "ms", Positive, Decimal, &[2, 1], Some(2)),
        ];
        let mut num = ParsedNumber::new();
        for (n, &(input, output, sign, radix, digits, exponent)) in CASES.iter().enumerate() {
            let offset: u32 = (1 + n as u32) * 100;
            let in_span = Span {
                start: Pos(offset),
                end: Pos(offset + input.len() as u32),
            };
            match num.parse(input, in_span) {
                Err((e, _)) => {
                    success = false;
                    eprintln!("Test case {} failed:", n);
                    eprintln!("    Input: {:?}", input);
                    eprintln!("    Error: {:?}", e);
                }
                Ok(rest) => {
                    if rest != output
                        || num.sign != sign
                        || num.radix != radix
                        || num.digits != digits
                        || num.exponent != exponent
                    {
                        success = false;
                        eprintln!("Test case {} failed:", n);
                        eprintln!("    Input: {:?}", input);
                        if rest != output {
                            eprintln!("    Output: {:?}, expected {:?}", rest, output);
                        }
                        if num.sign != sign {
                            eprintln!("    Sign: {:?}, expected {:?}", num.sign, sign);
                        }
                        if num.radix != radix {
                            eprintln!("    Radix: {}, expected {}", num.radix as u8, radix as u8);
                        }
                        if num.digits != digits {
                            eprintln!("    Digits: {:?}, expected {:?}", num.digits, digits);
                        }
                        if num.exponent != exponent {
                            eprintln!("    Exponent: {:?}, expected {:?}", num.exponent, exponent);
                        }
                    }
                }
            }
        }
        if !success {
            eprintln!();
            panic!("failed");
        }
    }
}
