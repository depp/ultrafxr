use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

const NOTE_NAMES: [&'static str; 12] = [
    "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b",
];

/// A MIDI note value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Note(pub u8);

impl Note {
    pub fn octave(&self) -> i32 {
        self.0 as i32 / 12 - 1
    }
    pub fn chromaticity(&self) -> i32 {
        self.0 as i32 % 12
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            NOTE_NAMES[self.chromaticity() as usize],
            self.octave()
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseNoteError {
    CannotParse,
    UnknownNote,
    InvalidAccidentals,
    OutOfRange,
}

impl FromStr for Note {
    type Err = ParseNoteError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let mut value: i32 = match chars.next().ok_or(ParseNoteError::CannotParse)? {
            'c' | 'C' => 0,
            'd' | 'D' => 2,
            'e' | 'E' => 4,
            'f' | 'F' => 5,
            'g' | 'G' => 7,
            'a' | 'A' => 9,
            'b' | 'B' => 11,
            _ => return Err(ParseNoteError::UnknownNote),
        };
        let mut rest = chars.as_str();
        match chars.next() {
            Some('b') => {
                let mut flats: i32 = 1;
                loop {
                    rest = chars.as_str();
                    match chars.next() {
                        Some('b') if flats < 3 => flats += 1,
                        Some('b') | Some('#') => return Err(ParseNoteError::InvalidAccidentals),
                        _ => break,
                    }
                }
                value -= flats;
            }
            Some('#') => {
                let mut sharps: i32 = 1;
                loop {
                    rest = chars.as_str();
                    match chars.next() {
                        Some('#') if sharps < 3 => sharps += 1,
                        Some('b') | Some('#') => return Err(ParseNoteError::InvalidAccidentals),
                        _ => break,
                    }
                }
                value += sharps;
            }
            _ => (),
        }
        let octave = match rest.parse::<i32>() {
            Ok(n) => n,
            Err(_) => return Err(ParseNoteError::CannotParse),
        };
        if octave < -2 || octave > 20 {
            return Err(ParseNoteError::OutOfRange);
        }
        value += (octave + 1) * 12;
        match u8::try_from(value) {
            Ok(n) => Ok(Note(n)),
            Err(_) => Err(ParseNoteError::OutOfRange),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Note;

    #[test]
    fn octave() {
        assert_eq!(Note(0).octave(), -1);
        assert_eq!(Note(11).octave(), -1);
        assert_eq!(Note(12).octave(), 0);
        assert_eq!(Note(59).octave(), 3);
        assert_eq!(Note(60).octave(), 4);
        assert_eq!(Note(61).octave(), 4);
    }

    #[test]
    fn chromaticity() {
        assert_eq!(Note(0).chromaticity(), 0);
        assert_eq!(Note(1).chromaticity(), 1);
        assert_eq!(Note(12).chromaticity(), 0);
        assert_eq!(Note(59).chromaticity(), 11);
        assert_eq!(Note(60).chromaticity(), 0);
        assert_eq!(Note(61).chromaticity(), 1);
    }

    #[test]
    fn format() {
        assert_eq!(Note(0).to_string(), "c-1");
        assert_eq!(Note(1).to_string(), "c#-1");
        assert_eq!(Note(12).to_string(), "c0");
        assert_eq!(Note(59).to_string(), "b3");
        assert_eq!(Note(60).to_string(), "c4");
        assert_eq!(Note(61).to_string(), "c#4");
    }

    #[test]
    fn parse() {
        for n in 0..=255 {
            let n = Note(n);
            assert_eq!(Ok(n), n.to_string().parse::<Note>());
        }
    }
}
