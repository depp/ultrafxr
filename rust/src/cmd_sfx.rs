use crate::consolelogger::ConsoleLogger;
use crate::note::Note;
use crate::parseargs::{Arg, Args, UsageError};
use crate::parser::{ParseResult, Parser};
use crate::token::Tokenizer;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Command {
    pub input: OsString,
    pub write_wav: bool,
    pub play: bool,
    pub notes: Option<Vec<Note>>,
    pub tempo: Option<f32>,
    pub gate: Option<f32>,
    pub disassemble: bool,
    pub do_loop: bool,
    pub verbose: bool,
}

fn parse_notes(arg: &str) -> Option<Vec<Note>> {
    let mut result = Vec::new();
    for s in arg.split(',') {
        result.push(s.parse::<Note>().ok()?);
    }
    Some(result)
}

impl Command {
    pub fn from_args(args: env::ArgsOs) -> Result<Command, UsageError> {
        let mut input = None;
        let mut write_wav = false;
        let mut play = false;
        let mut notes = None;
        let mut tempo = None;
        let mut gate = None;
        let mut disassemble = false;
        let mut do_loop = false;
        let mut verbose = false;
        let mut args = Args::from_args(args);
        loop {
            args = match args.next()? {
                Arg::End => break,
                Arg::Positional(value, rest) => {
                    if input.is_some() {
                        return Err(UsageError::UnexpectedArgument { arg: value });
                    }
                    input = Some(value);
                    rest
                }
                Arg::Named(option) => match option.name() {
                    "write-wav" => {
                        write_wav = true;
                        option.no_value()?.1
                    }
                    "play" => {
                        play = true;
                        option.no_value()?.1
                    }
                    "notes" => {
                        let (_, value, rest) = option.parse_str(parse_notes)?;
                        notes = Some(value);
                        rest
                    }
                    "tempo" => {
                        let (_, value, rest) = option.parse_str(|s| s.parse::<f32>().ok())?;
                        tempo = Some(value);
                        rest
                    }
                    "gate" => {
                        let (_, value, rest) = option.parse_str(|s| s.parse::<f32>().ok())?;
                        gate = Some(value);
                        rest
                    }
                    "disassemble" => {
                        disassemble = true;
                        option.no_value()?.1
                    }
                    "loop" => {
                        do_loop = true;
                        option.no_value()?.1
                    }
                    "verbose" => {
                        verbose = true;
                        option.no_value()?.1
                    }
                    _ => return Err(option.unknown()),
                },
            };
        }
        let input = match input {
            Some(x) => x,
            None => {
                return Err(UsageError::MissingArgument {
                    name: "file".to_owned(),
                })
            }
        };
        Ok(Command {
            input,
            write_wav,
            play,
            notes,
            tempo,
            gate,
            disassemble,
            do_loop,
            verbose,
        })
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut text = Vec::new();
        let mut file = File::open(&self.input)?;
        let filename = match self.input.to_str() {
            Some(s) => s.to_string(),
            None => format!("{:?}", self.input),
        };
        file.read_to_end(&mut text)?;
        drop(file);
        let mut toks = Tokenizer::new(text.as_ref())?;
        let mut parser = Parser::new();
        let mut err_handler = ConsoleLogger::from_text(filename.as_ref(), text.as_ref());
        loop {
            match parser.parse(&mut err_handler, &mut toks) {
                ParseResult::None => break,
                ParseResult::Incomplete => {
                    parser.finish(&mut err_handler);
                    break;
                }
                ParseResult::Error => break,
                ParseResult::Value(expr) => {
                    println!("Expr: {:?}", expr);
                }
            }
        }
        Ok(())
    }
}
