use std::env;
use std::ffi::OsString;
use std::fmt;

#[derive(Debug, Clone)]
pub enum UsageError {
    UnknownOption { option: String },
    OptionMissingParameter { option: String },
    OptionUnexpectedParameter { option: String },
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UsageError::*;
        match self {
            UnknownOption { option } => write!(f, "unknown option -{}", option),
            OptionMissingParameter { option } => write!(f, "option -{} requires parameter", option),
            OptionUnexpectedParameter { option } => write!(f, "unknown option {:?}", option),
        }
    }
}

pub struct Args {
    args: env::ArgsOs,
    allow_options: bool,
}

impl Args {
    pub fn args() -> Self {
        Args {
            args: env::args_os(),
            allow_options: true,
        }
    }
    pub fn next(self) -> Arg {
        let Args {
            mut args,
            allow_options,
        } = self;
        let arg = match args.next() {
            None => return Arg::End,
            Some(arg) => arg,
        };
        if allow_options {
            let arg_str = arg.to_string_lossy();
            if arg_str.len() >= 2 && arg_str.starts_with("-") {
                if arg_str == "--" {
                    return match args.next() {
                        None => Arg::End,
                        Some(arg) => Arg::Positional(
                            arg,
                            Args {
                                args,
                                allow_options: false,
                            },
                        ),
                    };
                }
                let body = if arg_str.starts_with("--") {
                    &arg_str[2..]
                } else {
                    &arg_str[1..]
                };
                let (option, option_value) = match body.find('=') {
                    None => (String::from(body), None),
                    Some(idx) => (
                        String::from(&body[..idx]),
                        Some(OsString::from(&body[idx + 1..])),
                    ),
                };
                return Arg::Named(NamedArgument {
                    option,
                    option_value,
                    args,
                });
            }
        }
        Arg::Positional(
            arg,
            Args {
                args,
                allow_options,
            },
        )
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
    pub fn value(self) -> Result<(OsString, Args), UsageError> {
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
            value,
            Args {
                args,
                allow_options: true,
            },
        ))
    }
    pub fn no_value(self) -> Result<Args, UsageError> {
        let NamedArgument {
            option,
            option_value,
            args,
        } = self;
        if option_value.is_some() {
            return Err(UsageError::OptionUnexpectedParameter { option });
        }
        Ok(Args {
            args,
            allow_options: true,
        })
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
