#[macro_use]
mod consolelogger;

mod cmd_sfx;
mod color;
mod error;
mod evaluate;
mod note;
mod number;
mod parser;
mod sexpr;
mod signal;
mod sourcepos;
mod sourceprint;
mod sourcetext;
mod token;
mod utf8;
mod wave;

#[allow(dead_code)]
mod parseargs;

#[allow(dead_code)]
mod units;

#[allow(dead_code)]
mod rand;

#[cfg(test)]
mod test;

use crate::error::Failed;
use std::env;
use std::process;

fn main() {
    let mut args = env::args_os();
    // Discard program name.
    args.next();
    let cmd = match cmd_sfx::Command::from_args(args) {
        Ok(c) => c,
        Err(e) => {
            error!("{}", e);
            process::exit(64);
        }
    };
    match cmd.run() {
        Ok(_) => (),
        Err(Failed) => process::exit(1),
    }
}
