mod error;
mod sexpr;
mod sourcepos;
mod token;

use error::ErrorHandler;
use sexpr::SExpr;
use sourcepos::{Pos, Span};
use std::fmt;
use std::str::from_utf8;
use token::{Token, Tokenizer, Type};

const TEXT: &'static str = "(abc def) (ghi)";

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
    let mut toks = Tokenizer::new(TEXT.as_bytes());
    loop {
        let tok = toks.next();
        print_token(&tok);
        if tok.ty == Type::End {
            break;
        }
    }
    let mut toks = Tokenizer::new(TEXT.as_bytes());
    let res = SExpr::parse(&mut StderrLogger {}, &mut toks);
    match res {
        Some(exprs) => {
            for expr in exprs.iter() {
                println!("{:?}", expr);
            }
        }
        None => {}
    }
}
