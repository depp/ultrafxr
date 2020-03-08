use crate::consolelogger::ConsoleLogger;
use crate::evaluate::evaluate_program;
use crate::note::Note;
use crate::parseargs::{Arg, Args, UsageError};
use crate::parser::{ParseResult, Parser};
use crate::token::Tokenizer;
use crate::wave;
use std::env;
use std::error::Error;
use std::f32;
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone)]
enum CError {
    ParseFailed,
    EvalFailed,
}

impl Display for CError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use CError::*;
        f.write_str(match self {
            ParseFailed => "parsing failed",
            EvalFailed => "evaluation failed",
        })
    }
}

impl Error for CError {}

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
    pub dump_syntax: bool,
    pub dump_graph: bool,
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
        let mut dump_syntax = false;
        let mut dump_graph = false;
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
                    "dump-syntax" => {
                        dump_syntax = true;
                        option.no_value()?.1
                    }
                    "dump-graph" => {
                        dump_graph = true;
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
            dump_syntax,
            dump_graph,
        })
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let filename = match self.input.to_str() {
            Some(s) => s.to_string(),
            None => format!("{:?}", self.input),
        };
        let text = {
            let mut text = Vec::new();
            let mut file = File::open(&self.input)?;
            file.read_to_end(&mut text)?;
            text
        };
        let mut err_handler = ConsoleLogger::from_text(filename.as_ref(), text.as_ref());
        let exprs = {
            let mut exprs = Vec::new();
            let mut toks = Tokenizer::new(text.as_ref())?;
            let mut parser = Parser::new();
            loop {
                match parser.parse(&mut err_handler, &mut toks) {
                    ParseResult::None => break,
                    ParseResult::Incomplete => {
                        parser.finish(&mut err_handler);
                        break;
                    }
                    ParseResult::Error => return Err(Box::new(CError::ParseFailed)),
                    ParseResult::Value(expr) => {
                        if self.dump_syntax {
                            eprintln!("Syntax: {}", expr.print());
                        }
                        exprs.push(expr);
                    }
                }
            }
            exprs
        };
        let (graph, root) = match evaluate_program(&mut err_handler, exprs.as_ref()) {
            Some(r) => r,
            None => return Err(Box::new(CError::EvalFailed)),
        };
        if self.dump_graph {
            let mut stdout = stdout();
            graph.dump(&mut stdout);
            writeln!(&mut stdout, "root = {:?}", root).unwrap();
        }
        if self.write_wav {
            let mut path = PathBuf::from(self.input.clone());
            if path.extension() == Some(OsStr::new("wav")) {
                panic!("refusing to overwrite input file");
            }
            path.set_extension("wav");
            let mut file = File::create(path)?;
            let mut writer = wave::Writer::from_stream(
                &mut file,
                &wave::Parameters {
                    channel_count: 1,
                    sample_rate: 48000,
                },
            );
            let mut buf = Vec::new();
            let w = f32::consts::PI * 440.0 * 1.0 / 48000.0;
            for i in 0..48000 {
                buf.push(((i as f32) * w).sin());
            }
            writer.write(&buf[..])?;
            writer.finish()?;
            file.sync_all()?;
        }
        Ok(())
    }
}
