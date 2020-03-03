use std::fmt;

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

/// Parse a metric prefix from the beginning of a string.
///
/// Returns the prefix's power of 10 and the remainder of the string after the
/// prefix. This function only parses prefixes for powers of 1000. The hecto
/// (h), deca (da), deci (d), and centi (c) prefixes are not recognized.
fn parse_prefix(text: &str) -> Option<(i32, &str)> {
    let mut chars = text.chars();
    let c = match chars.next() {
        None => return None,
        Some(c) => c,
    };
    // U+00B5: Micro Sign
    // U+03BC: Greek Small Letter Mu
    let exponent: i32 = match c {
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
    };
    Some((exponent, chars.as_str()))
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
            fn $id(n: i8) -> Self {
                let mut u: Units = Default::default();
                u.$id = n;
                u
            }
        )*
    };
}

impl Units {
    def_units!(volt, second, radian, decibel);

    fn hertz(n: i8) -> Self {
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
    pub fn parse(text: &str) -> Option<(Self, i32)> {
        if text.is_empty() {
            return Some(Default::default());
        }
        if let Some((exponent, rest)) = parse_prefix(text) {
            if let Some((true, units)) = Units::parse_without_prefix(rest) {
                return Some((units, exponent));
            }
        }
        if let Some((_, units)) = Units::parse_without_prefix(text) {
            return Some((units, 0));
        }
        None
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
            value: i8,
        ) -> fmt::Result {
            if value == 0 {
                Ok(())
            } else {
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
        write_unit(f, &mut has_text, "V", self.volt)?;
        write_unit(f, &mut has_text, "s", self.second)?;
        write_unit(f, &mut has_text, "rad", self.radian)?;
        write_unit(f, &mut has_text, "dB", self.decibel)?;
        if !has_text {
            f.write_str("1")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Units;

    #[test]
    fn display() {
        assert_eq!(Units::default().to_string(), "1");
        assert_eq!(Units::volt(1).to_string(), "V");
        assert_eq!(Units::second(1).to_string(), "s");
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
        let cases: &[(&'static str, Units, i32)] = &[
            ("", Default::default(), 0),
            ("V", Units::volt(1), 0),
            ("s", Units::second(1), 0),
            ("Hz", Units::hertz(1), 0),
            ("rad", Units::radian(1), 0),
            ("dB", Units::decibel(1), 0),
            ("mV", Units::volt(1), -3),
            ("kHz", Units::hertz(1), 3),
        ];
        for (n, &(input, units, exponent)) in cases.iter().enumerate() {
            let out = Units::parse(input);
            let expect = Some((units, exponent));
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
        const CASES: &'static [&'static str] = &[
            "v",   // Wrong case, should be V.
            "mdB", // Prefix not permitted, dB already has prefix.
            "kv",  // Wrong case, should be kV.
            "k",   // No units.
            "qV",  // Invalid prefix.
            "mS",  // Unknown units.
        ];
        for (n, &input) in CASES.iter().enumerate() {
            let out = Units::parse(input);
            if out.is_some() {
                success = false;
                eprintln!("Test {} failed:", n);
                eprintln!("    Input: {:?}", input);
                eprintln!("    Output:   {:?}", out);
                eprintln!("    Expected: None");
            }
        }
        if !success {
            eprintln!();
            panic!("failed");
        }
    }
}
