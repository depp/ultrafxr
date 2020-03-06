use crate::error::ErrorHandler;
use crate::sexpr::SExpr;
use crate::signal::graph::{Graph, SignalRef};
use crate::sourcepos::Span;
use crate::units::Units;

#[macro_use]
mod environment;

mod builtins;
mod envelope;

use environment::*;

/// Evaluate an audio synthesis program.
pub fn evaluate_program(
    err_handler: &mut dyn ErrorHandler,
    program: &[SExpr],
) -> Option<(Graph, SignalRef)> {
    // Break program into the leading forms and the last form. The last form is
    // considered to be the output, and must produce a value.
    let (last, first) = match program.split_last() {
        None => {
            err_handler.handle(Span::none(), "empty program");
            return None;
        }
        Some(x) => x,
    };
    let mut env = Env::new(err_handler, builtins::operators());
    for form in first.iter() {
        match env.evaluate(form).into_void() {
            EvalResult(_, Ok(())) => (),
            EvalResult(label, Err(e)) => match e {
                ValueError::Failed => (),
                _ => log_error!(env, label.pos, "invalid top-level statement: {}", e),
            },
        }
    }
    let signal = match env.evaluate(last).into_signal(Units::volt(1)) {
        EvalResult(_, Ok(sig)) => sig,
        EvalResult(label, Err(e)) => {
            match e {
                ValueError::Failed => (),
                _ => log_error!(env, label.pos, "invalid program body: {}", e),
            }
            return None;
        }
    };
    env.into_graph().map(|g| (g, signal))
}
