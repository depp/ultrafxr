use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;

#[derive(Debug, Clone)]
pub enum UsageError {
    InvalidArgument { arg: OsString },
    UnexpectedArgument { arg: OsString },
    MissingArgument { name: String },
    UnknownOption { option: String },
    OptionMissingParameter { option: String },
    OptionUnexpectedParameter { option: String },
    OptionInvalidValue { option: String, value: OsString },
    Custom { text: String },
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UsageError::*;
        match self {
            InvalidArgument { arg } => write!(f, "invalid argument {:?}", arg),
            UnexpectedArgument { arg } => write!(f, "unexpected argument {:?}", arg),
            MissingArgument { name } => write!(f, "missing argument <{}>", name),
            UnknownOption { option } => write!(f, "unknown option -{}", option),
            OptionMissingParameter { option } => write!(f, "option -{} requires parameter", option),
            OptionUnexpectedParameter { option } => write!(f, "unknown option {:?}", option),
            OptionInvalidValue { option, value } => {
                write!(f, "invalid value for -{}: {:?}", option, value)
            }
            Custom { text } => f.write_str(text),
        }
    }
}

fn is_arg_name(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
        _ => false,
    }
}

fn parse_arg(arg: OsString) -> Result<ParsedArg, UsageError> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let bytes = arg.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'-' {
        return Ok(ParsedArg::Positional(arg));
    }
    let body = if bytes[1] != b'-' {
        &bytes[1..]
    } else if bytes.len() == 2 {
        return Ok(ParsedArg::EndOfFlags);
    } else {
        &bytes[2..]
    };
    let (name, value) = match body.iter().position(|&c| c == b'=') {
        None => (body, None),
        Some(idx) => (&body[..idx], Some(&body[idx + 1..])),
    };
    if name.len() == 0
        || name[0] == b'-'
        || name[name.len() - 1] == b'-'
        || !name.iter().all(|&c| is_arg_name(c as char))
    {
        return Err(UsageError::InvalidArgument { arg });
    }
    let name = Vec::from(name);
    let name = unsafe { String::from_utf8_unchecked(name) };
    let value = value.map(|v| OsString::from_vec(Vec::from(v)));
    Ok(ParsedArg::Named(name, value))
}

enum ParsedArg {
    Positional(OsString),            // A positional argument.
    EndOfFlags,                      // The "--" argument.
    Named(String, Option<OsString>), // A named option -opt or -opt=value.
}

pub struct Args {
    args: env::ArgsOs,
    allow_options: bool,
}

impl Args {
    pub fn from_args(args: env::ArgsOs) -> Self {
        Args {
            args,
            allow_options: true,
        }
    }
    pub fn next(self) -> Result<Arg, UsageError> {
        let Args {
            mut args,
            allow_options,
        } = self;
        let arg = match args.next() {
            None => return Ok(Arg::End),
            Some(arg) => arg,
        };
        let arg = if allow_options {
            parse_arg(arg)?
        } else {
            ParsedArg::Positional(arg)
        };
        Ok(match arg {
            ParsedArg::Positional(arg) => Arg::Positional(
                arg,
                Args {
                    args,
                    allow_options,
                },
            ),
            ParsedArg::EndOfFlags => match args.next() {
                None => Arg::End,
                Some(arg) => Arg::Positional(
                    arg,
                    Args {
                        args,
                        allow_options: false,
                    },
                ),
            },
            ParsedArg::Named(name, value) => Arg::Named(NamedArgument {
                option: name,
                option_value: value,
                args,
            }),
        })
    }
}

pub struct NamedArgument {
    option: String,
    option_value: Option<OsString>,
    args: env::ArgsOs,
}

impl NamedArgument {
    pub fn name(&self) -> &str {
        self.option.as_ref()
    }
    pub fn value_osstr(self) -> Result<(String, OsString, Args), UsageError> {
        let NamedArgument {
            option,
            option_value,
            mut args,
        } = self;
        let value = match option_value {
            None => match args.next() {
                None => return Err(UsageError::OptionMissingParameter { option }),
                Some(value) => value,
            },
            Some(value) => value,
        };
        Ok((
            option,
            value,
            Args {
                args,
                allow_options: true,
            },
        ))
    }
    pub fn parse_osstr<T, F: FnOnce(&OsStr) -> Option<T>>(
        self,
        f: F,
    ) -> Result<(String, T, Args), UsageError> {
        let (option, value, rest) = self.value_osstr()?;
        match f(value.as_ref()) {
            None => Err(UsageError::OptionInvalidValue { option, value }),
            Some(x) => Ok((option, x, rest)),
        }
    }
    pub fn value_str(self) -> Result<(String, String, Args), UsageError> {
        self.parse_osstr(|s| s.to_str().map(String::from))
    }
    pub fn parse_str<T, F: FnOnce(&str) -> Option<T>>(
        self,
        f: F,
    ) -> Result<(String, T, Args), UsageError> {
        self.parse_osstr(|s| s.to_str().and_then(|s| f(String::from(s).as_str())))
    }
    pub fn no_value(self) -> Result<(String, Args), UsageError> {
        let NamedArgument {
            option,
            option_value,
            args,
        } = self;
        if option_value.is_some() {
            return Err(UsageError::OptionUnexpectedParameter { option });
        }
        Ok((
            option,
            Args {
                args,
                allow_options: true,
            },
        ))
    }
    pub fn unknown(self) -> UsageError {
        UsageError::UnknownOption {
            option: self.option,
        }
    }
}

pub enum Arg {
    Positional(OsString, Args),
    Named(NamedArgument),
    End,
}
