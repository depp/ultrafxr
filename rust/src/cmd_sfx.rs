use crate::consolelogger::ConsoleLogger;
use crate::error::Failed;
use crate::evaluate::evaluate_program;
use crate::note::Note;
use crate::parseargs::{Arg, Args, UsageError};
use crate::parser::{ParseResult, Parser};
use crate::shell::quote_os;
use crate::signal::graph::{Graph, SignalRef};
use crate::signal::program::{Input as PInput, Parameters, Program};
use crate::token::Tokenizer;
use crate::wave;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{stdout, Error as IOError, Read, Write};
use std::path::PathBuf;

const DEFAULT_SAMPLE_RATE: u32 = 48000;
const MIN_SAMPLE_RATE: u32 = 8000;
const MAX_SAMPLE_RATE: u32 = 192000;
const DEFAULT_BUFFER_SIZE: usize = 1024;
const MIN_BUFFER_SIZE: usize = 32;
const MAX_BUFFER_SIZE: usize = 8192;

#[derive(Debug, Clone)]
pub enum Input {
    File(OsString),
    String(String),
}

#[derive(Debug, Clone)]
pub struct File {
    pub input: Input,
    pub output_wave: Option<OsString>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub files: Vec<File>,
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

fn unwrap_write<T>(filename: &str, result: Result<T, IOError>) -> Result<T, Failed> {
    match result {
        Ok(x) => Ok(x),
        Err(e) => {
            error!("could not write {}: {}", filename, e);
            Err(Failed)
        }
    }
}

impl Command {
    pub fn from_args(args: env::ArgsOs) -> Result<Command, UsageError> {
        let mut inputs = Vec::new();
        let mut script = None;
        let mut do_write_wave = false;
        let mut wave_file = None;
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
                    inputs.push(value);
                    rest
                }
                Arg::Named(option) => match option.name() {
                    "write-wav" => {
                        do_write_wave = true;
                        option.no_value()?.1
                    }
                    "wav-out" => {
                        let (_, value, rest) = option.value_osstr()?;
                        wave_file = Some(value);
                        rest
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
                    "script" => {
                        let (_, value, rest) = option.value_str()?;
                        script = Some(value);
                        rest
                    }
                    _ => return Err(option.unknown()),
                },
            };
        }
        let mut files = Vec::new();
        match script {
            Some(input) => {
                if !inputs.is_empty() {
                    return Err(UsageError::Custom {
                        text: "cannot specify both -script and <file>".to_string(),
                    });
                }
                let input = Input::String(input);
                let mut output_wave = wave_file;
                if output_wave.is_none() && do_write_wave {
                    output_wave = Some(OsString::from("ultrafxr.wav"));
                }
                files.push(File { input, output_wave });
            }
            None => {
                if inputs.is_empty() {
                    return Err(UsageError::Custom {
                        text: format!("no inputs"),
                    });
                }
                for input in inputs.drain(..) {
                    files.push(File {
                        input: Input::File(input),
                        output_wave: None,
                    });
                }
                if do_write_wave {
                    match wave_file {
                        Some(path) => {
                            if files.len() != 1 {
                                return Err(UsageError::Custom {
                                    text: "-wav-output cannot be used with multiple inputs"
                                        .to_string(),
                                });
                            }
                            files[0].output_wave = Some(path);
                        }
                        None => {
                            for file in files.iter_mut() {
                                let path = match &file.input {
                                    &Input::File(ref path) => path,
                                    _ => panic!("expected file"),
                                };
                                let mut path = PathBuf::from(path.clone());
                                if path.extension() == Some(OsStr::new("wav")) {
                                    return Err(UsageError::Custom {
                                        text: format!(
                                            "refusing to overwrite input file {}",
                                            quote_os(&path)
                                        ),
                                    });
                                }
                                path.set_extension("wav");
                                file.output_wave = Some(OsString::from(path))
                            }
                        }
                    }
                }
            }
        }
        Ok(Command {
            files,
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
        for file in self.files.iter() {
            self.run_file(file)?;
        }
        Ok(())
    }

    fn run_file(&self, file: &File) -> Result<(), Failed> {
        let (filename, text) = self.read_input(file)?;
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
        match file.output_wave {
            Some(ref path) => self.write_wave(path, &graph, root)?,
            None => {}
        }
        Ok(())
    }

    /// Read the input file and return its name and its contents.
    fn read_input(&self, file: &File) -> Result<(String, Box<[u8]>), Failed> {
        match file.input {
            Input::File(ref path) => {
                let filename = quote_os(path);
                let mut text = Vec::new();
                match fs::File::open(path).and_then(|mut f| f.read_to_end(&mut text)) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("could not read {}: {}", filename, e);
                        return Err(Failed);
                    }
                }
                Ok((filename, Box::from(text)))
            }
            Input::String(ref s) => Ok(("<arg>".to_string(), Box::from(s.as_bytes()))),
        }
    }

    /// Write output wave file.
    fn write_wave(&self, path: &OsStr, graph: &Graph, signal: SignalRef) -> Result<(), Failed> {
        let filename = quote_os(path);
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
        let buffer_size = match self.buffer_size {
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
        let note = self
            .notes
            .as_ref()
            .and_then(|x| x.first().copied())
            .unwrap_or(Note(60));
        let program = Program::new(
            &graph,
            signal,
            &Parameters {
                sample_rate: sample_rate as f64,
                buffer_size,
            },
        );
        let mut program = match program {
            Ok(p) => p,
            Err(e) => {
                error!("could not create program: {}", e);
                return Err(Failed);
            }
        };
        let mut file = match fs::File::create(&path) {
            Ok(file) => file,
            Err(e) => {
                error!("could not create {}: {}", filename, e);
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
        let mut pos: usize = 0;
        let end = sample_rate as usize;
        loop {
            let output = program.render(&PInput {
                gate: if pos < end && end - pos < buffer_size {
                    Some(end - pos)
                } else {
                    None
                },
                note: note.0 as f32,
            });
            let output = match output {
                Some(x) => x,
                None => break,
            };
            pos += output.len();
            unwrap_write(&filename, writer.write(output))?;
        }
        unwrap_write(&filename, writer.finish())?;
        unwrap_write(&filename, file.sync_all())
    }
}
