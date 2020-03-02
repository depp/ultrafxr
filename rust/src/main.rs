mod cmd_sfx;
mod color;
mod consolelogger;
mod error;
mod note;
mod parser;
mod sexpr;
mod sourcepos;
mod sourceprint;
mod sourcetext;
mod token;
mod utf8;

#[allow(dead_code)]
mod parseargs;

#[allow(dead_code)]
mod units;

#[cfg(test)]
mod test;

use consolelogger::write_diagnostic;
use error::Severity;
use std::env;
use std::io::stderr;
use std::process;

fn main() {
    let mut stderr = stderr();
    let mut args = env::args_os();
    // Discard program name.
    args.next();
    let cmd = match cmd_sfx::Command::from_args(args) {
        Ok(c) => c,
        Err(e) => {
            write_diagnostic(&mut stderr, Severity::Error, &e).unwrap();
            process::exit(64);
        }
    };
    match cmd.run() {
        Ok(_) => (),
        Err(e) => {
            write_diagnostic(&mut stderr, Severity::Error, &e).unwrap();
            process::exit(1);
        }
    }
}
