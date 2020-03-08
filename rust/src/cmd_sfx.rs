use crate::consolelogger::ConsoleLogger;
use crate::error::Failed;
use crate::evaluate::evaluate_program;
use crate::note::Note;
use crate::parseargs::{Arg, Args, UsageError};
use crate::parser::{ParseResult, Parser};
use crate::signal::graph::{Graph, SignalRef};
use crate::token::Tokenizer;
use crate::wave;
use std::env;
use std::f32;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{stdout, Error as IOError, Read, Write};
use std::path::{Path, PathBuf};

const DEFAULT_SAMPLE_RATE: u32 = 48000;
const MIN_SAMPLE_RATE: u32 = 8000;
const MAX_SAMPLE_RATE: u32 = 192000;
const DEFAULT_BUFFER_SIZE: usize = 1024;
const MIN_BUFFER_SIZE: usize = 32;
const MAX_BUFFER_SIZE: usize = 8192;

#[derive(Debug, Clone)]
pub struct Command {
    pub input: OsString,
    pub do_write_wave: bool,
    pub play: bool,
    pub notes: Option<Vec<Note>>,
    pub tempo: Option<f32>,
    pub gate: Option<f32>,
    pub disassemble: bool,
    pub do_loop: bool,
    pub verbose: bool,
    pub dump_syntax: bool,
    pub dump_graph: bool,
    pub sample_rate: Option<u32>,
    pub buffer_size: Option<usize>,
}

fn parse_notes(arg: &str) -> Option<Vec<Note>> {
    let mut result = Vec::new();
    for s in arg.split(',') {
        result.push(s.parse::<Note>().ok()?);
    }
    Some(result)
}

fn unwrap_write<T>(path: &Path, result: Result<T, IOError>) -> Result<T, Failed> {
    match result {
        Ok(x) => Ok(x),
        Err(e) => {
            error!("could not write {}: {}", path.to_string_lossy(), e);
            Err(Failed)
        }
    }
}

impl Command {
    pub fn from_args(args: env::ArgsOs) -> Result<Command, UsageError> {
        let mut input = None;
        let mut do_write_wave = false;
        let mut play = false;
        let mut notes = None;
        let mut tempo = None;
        let mut gate = None;
        let mut disassemble = false;
        let mut do_loop = false;
        let mut verbose = false;
        let mut dump_syntax = false;
        let mut dump_graph = false;
        let mut sample_rate = None;
        let mut buffer_size = None;
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
                        do_write_wave = true;
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
                    "sample-rate" => {
                        let (_, value, rest) = option.parse_str(|s| s.parse::<u32>().ok())?;
                        sample_rate = Some(value);
                        rest
                    }
                    "buffer-size" => {
                        let (_, value, rest) = option.parse_str(|s| s.parse::<usize>().ok())?;
                        buffer_size = Some(value);
                        rest
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
            do_write_wave,
            play,
            notes,
            tempo,
            gate,
            disassemble,
            do_loop,
            verbose,
            dump_syntax,
            dump_graph,
            sample_rate,
            buffer_size,
        })
    }

    pub fn run(&self) -> Result<(), Failed> {
        let filename = match self.input.to_str() {
            Some(s) => s.to_string(),
            None => format!("{:?}", self.input),
        };
        let text = match self.read_input() {
            Ok(text) => text,
            Err(e) => {
                error!("could not read {}: {}", filename, e);
                return Err(Failed);
            }
        };
        let mut err_handler = ConsoleLogger::from_text(filename.as_ref(), text.as_ref());
        let exprs = {
            let mut exprs = Vec::new();
            let mut toks = match Tokenizer::new(text.as_ref()) {
                Ok(toks) => toks,
                Err(e) => {
                    error!("could not parse {}: {}", filename, e);
                    return Err(Failed);
                }
            };
            let mut parser = Parser::new();
            loop {
                match parser.parse(&mut err_handler, &mut toks) {
                    ParseResult::None => break,
                    ParseResult::Incomplete => {
                        parser.finish(&mut err_handler);
                        break;
                    }
                    ParseResult::Error => return Err(Failed),
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
        let (graph, root) = evaluate_program(&mut err_handler, exprs.as_ref())?;
        if self.dump_graph {
            let mut stdout = stdout();
            graph.dump(&mut stdout);
            writeln!(&mut stdout, "root = {:?}", root).unwrap();
        }
        if self.do_write_wave {
            self.write_wave(&graph, root)?;
        }
        Ok(())
    }

    /// Read the input file and return its contents.
    fn read_input(&self) -> Result<Box<[u8]>, IOError> {
        let mut text = Vec::new();
        let mut file = File::open(&self.input)?;
        file.read_to_end(&mut text)?;
        Ok(Box::from(text))
    }

    /// Write output wave file.
    fn write_wave(&self, _graph: &Graph, _signal: SignalRef) -> Result<(), Failed> {
        let sample_rate = match self.sample_rate {
            Some(rate) => {
                if rate < MIN_SAMPLE_RATE {
                    error!(
                        "sample rate {} is too low, acceptable rates are {}-{}",
                        rate, MIN_SAMPLE_RATE, MAX_SAMPLE_RATE
                    );
                    return Err(Failed);
                } else if rate > MAX_SAMPLE_RATE {
                    error!(
                        "sample rate {} is too high, acceptable rates are {}-{}",
                        rate, MIN_SAMPLE_RATE, MAX_SAMPLE_RATE
                    );
                    return Err(Failed);
                } else {
                    rate
                }
            }
            None => DEFAULT_SAMPLE_RATE,
        };
        let _buffer_size = match self.buffer_size {
            Some(size) => {
                if size < MIN_BUFFER_SIZE {
                    warning!("buffer size {} is too low, using {}", size, MIN_BUFFER_SIZE);
                    MIN_BUFFER_SIZE
                } else if size > MAX_BUFFER_SIZE {
                    warning!(
                        "buffer size {} is too high, using {}",
                        size,
                        MAX_BUFFER_SIZE
                    );
                    MAX_BUFFER_SIZE
                } else {
                    let nsize = size.next_power_of_two();
                    if nsize != size {
                        warning!(
                            "buffer size {} is not a power of two, using {}",
                            size,
                            nsize
                        );
                    }
                    nsize
                }
            }
            None => DEFAULT_BUFFER_SIZE,
        };
        let mut path = PathBuf::from(self.input.clone());
        if path.extension() == Some(OsStr::new("wav")) {
            error!(
                "refusing to overwrite input file {}",
                self.input.to_string_lossy(),
            );
            return Err(Failed);
        }
        path.set_extension("wav");
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(e) => {
                error!("could not create {}: {}", path.to_string_lossy(), e);
                return Err(Failed);
            }
        };
        let mut writer = wave::Writer::from_stream(
            &mut file,
            &wave::Parameters {
                channel_count: 1,
                sample_rate,
            },
        );
        let mut buf = Vec::new();
        let w = 2.0 * f32::consts::PI * 440.0 / sample_rate as f32;
        for i in 0..48000 {
            buf.push(((i as f32) * w).sin());
        }
        unwrap_write(&path, writer.write(&buf[..]))?;
        unwrap_write(&path, writer.finish())?;
        unwrap_write(&path, file.sync_all())
    }
}
