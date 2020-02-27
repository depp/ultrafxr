mod cmd_sfx;
mod color;
mod error;
mod note;
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
use std::env;
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

fn main() {
    let mut args = env::args_os();
    args.next();
    match cmd_sfx::Command::from_args(args) {
        Ok(c) => println!("cmd = {:?}", c),
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
