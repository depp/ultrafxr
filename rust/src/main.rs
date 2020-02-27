mod color;
mod error;
mod parseargs;
mod sexpr;
mod sourcepos;
mod sourceprint;
mod sourcetext;
mod token;
mod utf8;

#[cfg(test)]
mod test;

use color::{Style, StyleFlag};
use error::ErrorHandler;
use sexpr::{ParseResult, Parser};
use sourcepos::{HasPos, Pos, Span};
use sourceprint::print_source;
use sourcetext::SourceText;
use std::fmt;
use std::io::stdout;
use std::process;
use std::str::from_utf8;
use token::{Token, Tokenizer, Type};

const TEXT: &'static [u8] = b"(abc def) (ghi) '";

// Print a token to stdout for debugging.
fn print_token(tok: &Token) {
    let Token { ty, pos, text } = tok;
    let Pos(off) = pos;
    let utf8 = from_utf8(text);
    let dtext: &dyn fmt::Debug = match utf8 {
        Ok(ref s) => s,
        Err(_) => &text,
    };
    println!("{:4} {:?} {:?}", off, ty, dtext);
}

struct StderrLogger;

impl ErrorHandler for StderrLogger {
    fn handle(&mut self, _pos: Span, message: &str) {
        eprintln!("Error: {}", message);
    }
}

fn parse_args() -> Result<(), parseargs::UsageError> {
    use parseargs::{Arg, Args};
    let mut args = Args::args();
    loop {
        args = match args.next()? {
            Arg::End => break,
            Arg::Positional(value, rest) => {
                println!("Positional: {:?}", value);
                rest
            }
            Arg::Named(option) => match option.name() {
                "flag" => {
                    let (_, rest) = option.no_value()?;
                    println!("Flag");
                    rest
                }
                "osstr" => {
                    let (_, value, rest) = option.value_osstr()?;
                    println!("OsStr: {:?}", value);
                    rest
                }
                "str" => {
                    let (_, value, rest) = option.value_str()?;
                    println!("Str: {:?}", value);
                    rest
                }
                "int" => {
                    let (_, value, rest) = option.parse_str(|s| s.parse::<i32>().ok())?;
                    println!("Int: {:?}", value);
                    rest
                }
                _ => return Err(option.unknown()),
            },
        }
    }
    Ok(())
}

fn main() {
    match parse_args() {
        Ok(_) => (),
        Err(e) => eprintln!("Error: {}", e),
    }
    use StyleFlag::*;
    println!(
        "{}red {}green bold{}",
        Style(&[FgRed]),
        Style(&[FgGreen, Bold]),
        Style(&[Reset])
    );
    let mut toks = match Tokenizer::new(TEXT) {
        Ok(toks) => toks,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    let src_text = SourceText::new(TEXT);
    println!("pos(1): {:?}", src_text.pos(Pos(1)));
    println!("line(0): {:?}", src_text.line(0));
    let mut out = stdout();
    loop {
        let tok = toks.next();
        print_token(&tok);
        match src_text.span(tok.source_pos()) {
            Some(span) => match print_source(&mut out, &src_text, &span) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            },
            None => (),
        }
        if tok.ty == Type::End {
            break;
        }
    }
    toks.rewind();
    let mut parser = Parser::new();
    let mut err_handler = StderrLogger {};
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
}
