use crate::sourcepos::Span;
use std::fmt;

/// An error from an operation on units.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnitError {
    Overflow,
}

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnitError::*;
        f.write_str(match self {
            Overflow => "units too large, operation overflowed",
        })
    }
}

/// An error from parsing units.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnknownPrefix,
    UnknownUnits,
    PrefixNotAllowed,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseError::*;
        f.write_str(match *self {
            UnknownPrefix => "unknown prefix",
            UnknownUnits => "unknown units",
            PrefixNotAllowed => "metric prefix not allowed with units",
        })
    }
}

/// Parse a metric prefix. Returns the prefix's power of 10. Only powers of 1000
/// are recognized; so hecto (h), deca (da), deci (d), and centi (c) are
/// ignored.
fn parse_prefix(c: char) -> Option<i32> {
    // U+00B5: Micro Sign
    // U+03BC: Greek Small Letter Mu
    Some(match c {
        'y' => -24,
        'z' => -21,
        'a' => -18,
        'f' => -15,
        'p' => -12,
        'n' => -9,
        'u' | '\u{b5}' | '\u{3bc}' => -6,
        'm' => -3,
        'k' => 3,
        'M' => 6,
        'G' => 9,
        'T' => 12,
        'P' => 15,
        'E' => 18,
        'Z' => 21,
        'Y' => 24,
        _ => return None,
    })
}

/// Units associated with a quantity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Units {
    pub volt: i8,
    pub second: i8,
    pub radian: i8,
    pub decibel: i8,
}

macro_rules! def_units {
    ($($id:ident),*) => {
        $(
            pub fn $id(n: i8) -> Self {
                let mut u: Units = Default::default();
                u.$id = n;
                u
            }
        )*
    };
}

impl Units {
    /// True if this is the scalar unit, i.e., dimensionless or unitless.
    pub fn is_scalar(&self) -> bool {
        self == &Units::scalar()
    }

    /// Return the scalar (dimensionless, unitless) units.
    pub fn scalar() -> Self {
        Default::default()
    }

    def_units!(volt, second, radian, decibel);

    pub fn hertz(n: i8) -> Self {
        Units::second(-n)
    }

    /// True if this is a dimensionless scalar value, i.e., a scalar.
    pub fn dimensionless(&self) -> bool {
        *self == Default::default()
    }

    /// Multiplies two units.
    pub fn multiply(&self, other: &Units) -> Result<Self, UnitError> {
        let (volt, o1) = self.volt.overflowing_add(other.volt);
        let (second, o2) = self.second.overflowing_add(other.second);
        let (radian, o3) = self.volt.overflowing_add(other.radian);
        let (decibel, o4) = self.volt.overflowing_add(other.decibel);
        if o1 || o2 || o3 || o4 {
            Err(UnitError::Overflow)
        } else {
            Ok(Units {
                volt,
                second,
                radian,
                decibel,
            })
        }
    }

    /// Parse units with metric prefix.
    ///
    /// Returns the units and the exponent for the metric prefix used. For
    /// example, "ms" will parse as (second, -3), "kV" will parse as (volt, +3).
    pub fn parse(text: &str, pos: Span) -> Result<(Span, Self, i32), (ParseError, Span)> {
        use ParseError::*;
        let mut chars = text.chars();
        let c = match chars.next() {
            None => return Ok((pos, Default::default(), 0)),
            Some(c) => c,
        };
        let exponent = parse_prefix(c);
        let rest = chars.as_str();
        let split_idx = text.len() - rest.len();
        if let Some((allow_prefix, units)) = Units::parse_without_prefix(rest) {
            let exponent = match exponent {
                Some(x) => x,
                None => return Err((UnknownPrefix, pos.sub_span(..split_idx))),
            };
            if !allow_prefix {
                return Err((PrefixNotAllowed, pos));
            }
            return Ok((pos.sub_span(split_idx..), units, exponent));
        }
        if exponent.is_some() && !rest.is_empty() {
            return Err((UnknownUnits, pos.sub_span(split_idx..)));
        }
        match Units::parse_without_prefix(text) {
            Some((_, units)) => Ok((pos, units, 0)),
            None => Err((UnknownUnits, pos)),
        }
    }

    /// Parse units without metric prefix.
    ///
    /// Returns true if the units are permitted to have a metric prefix.
    fn parse_without_prefix(text: &str) -> Option<(bool, Self)> {
        Some(match text {
            "V" => (true, Units::volt(1)),
            "s" => (true, Units::second(1)),
            "Hz" => (true, Units::second(-1)),
            "rad" => (true, Units::radian(1)),
            "dB" => (false, Units::decibel(1)),
            _ => return None,
        })
    }
}

impl fmt::Display for Units {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_unit(
            f: &mut fmt::Formatter<'_>,
            has_text: &mut bool,
            name: &'static str,
            invname: &'static str,
            value: i8,
        ) -> fmt::Result {
            let value = value as i32;
            if value == 0 {
                Ok(())
            } else {
                let (name, value) = if value < 0 && !invname.is_empty() {
                    (invname, -value)
                } else {
                    (name, value)
                };
                if *has_text {
                    f.write_str("*")?;
                }
                *has_text = true;
                f.write_str(name)?;
                if value == 1 {
                    Ok(())
                } else {
                    write!(f, "^{}", value)
                }
            }
        }
        let mut has_text = false;
        write_unit(f, &mut has_text, "V", "", self.volt)?;
        write_unit(f, &mut has_text, "s", "Hz", self.second)?;
        write_unit(f, &mut has_text, "rad", "", self.radian)?;
        write_unit(f, &mut has_text, "dB", "", self.decibel)?;
        if !has_text {
            f.write_str("scalar")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sourcepos::{Pos, Span};

    #[test]
    fn display() {
        assert_eq!(Units::default().to_string(), "scalar");
        assert_eq!(Units::volt(1).to_string(), "V");
        assert_eq!(Units::second(1).to_string(), "s");
        assert_eq!(Units::second(-1).to_string(), "Hz");
        assert_eq!(Units::radian(1).to_string(), "rad");
        assert_eq!(Units::decibel(1).to_string(), "dB");
        assert_eq!(
            Units {
                volt: 2,
                second: 1,
                radian: -1,
                decibel: 0,
            }
            .to_string(),
            "V^2*s*rad^-1"
        );
    }

    #[test]
    fn parse() {
        let mut success = true;
        let cases: &[(&'static str, Units, i32, u32, u32)] = &[
            ("", Default::default(), 0, 0, 0),
            ("V", Units::volt(1), 0, 0, 1),
            ("s", Units::second(1), 0, 0, 1),
            ("Hz", Units::hertz(1), 0, 0, 2),
            ("rad", Units::radian(1), 0, 0, 3),
            ("dB", Units::decibel(1), 0, 0, 2),
            ("mV", Units::volt(1), -3, 1, 2),
            ("kHz", Units::hertz(1), 3, 1, 3),
            ("\u{03BC}s", Units::second(1), -6, 2, 3),
        ];
        for (n, &(input, units, exponent, start, end)) in cases.iter().enumerate() {
            let offset: u32 = (1 + n as u32) * 100;
            let in_pos = Span {
                start: Pos(offset),
                end: Pos(offset + input.len() as u32),
            };
            let out = Units::parse(input, in_pos);
            let expect_pos = Span {
                start: Pos(offset + start),
                end: Pos(offset + end),
            };
            let expect = Ok((expect_pos, units, exponent));
            if out != expect {
                success = false;
                eprintln!("Test {} failed:", n);
                eprintln!("    Input: {:?}", input);
                eprintln!("    Output:   {:?}", out);
                eprintln!("    Expected: {:?}", expect);
            }
        }
        if !success {
            eprintln!();
            panic!("failed");
        }
    }

    #[test]
    fn parse_fail() {
        let mut success = true;
        use ParseError::*;
        const CASES: &'static [(&'static str, ParseError, u32, u32)] = &[
            ("v", UnknownUnits, 0, 1),       // Wrong case, should be V.
            ("mdB", PrefixNotAllowed, 0, 3), // Prefix not permitted, dB already has prefix.
            ("kv", UnknownUnits, 1, 2),      // Wrong case, should be kV.
            ("k", UnknownUnits, 0, 1),       // No units.
            ("qV", UnknownPrefix, 0, 1),     // Invalid prefix.
            ("mS", UnknownUnits, 1, 2),      // Unknown units.
        ];
        for (n, &(input, err, start, end)) in CASES.iter().enumerate() {
            let offset: u32 = (1 + n as u32) * 100;
            let in_pos = Span {
                start: Pos(offset),
                end: Pos(offset + input.len() as u32),
            };
            let out = Units::parse(input, in_pos);
            let expect_pos = Span {
                start: Pos(offset + start),
                end: Pos(offset + end),
            };
            let expect = Err((err, expect_pos));
            if out != expect {
                success = false;
                eprintln!("Test {} failed:", n);
                eprintln!("    Input: {:?}", input);
                eprintln!("    Output:   {:?}", out);
                eprintln!("    Expected: {:?}", expect);
            }
        }
        if !success {
            eprintln!();
            panic!("failed");
        }
    }
}
