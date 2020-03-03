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
}
