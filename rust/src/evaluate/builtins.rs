use super::environment::*;
use crate::graph::{ops, Node};
use crate::sexpr::SExpr;
use crate::sourcepos::{HasPos, Span};
use crate::units::Units;
use std::collections::hash_map::{HashMap, RandomState};
use std::convert::TryFrom;

pub fn operators() -> HashMap<&'static str, Operator, RandomState> {
    let mut map = HashMap::new();
    fn add(
        map: &mut HashMap<&'static str, Operator, RandomState>,
        name: &'static str,
        value: Operator,
    ) {
        if map.insert(name, value).is_some() {
            panic!("duplicate operator name: {:?}", name);
        }
    }
    const UNIMPL_MACROS: &'static [&'static str] = &[
        //
        "envelope",
    ];
    for &name in UNIMPL_MACROS.iter() {
        add(&mut map, name, Operator::Macro(None));
    }
    macro_rules! macros {
            ($($f:ident);*;) => {
                $(add(&mut map, stringify!($f), Operator::Macro(Some($f)));)*
            };
        }
    macro_rules! function {
        ($name:literal, !) => {
            add(&mut map, $name, Operator::Function(None))
        };
        ($name:literal, $func:ident) => {
            add(&mut map, $name, Operator::Function(Some($func)))
        };
    }
    macro_rules! functions {
            ($($name:literal => $val:tt),*,) => {
                $(function!($name, $val));*
            };
        }
    macros!(
        define;
    );
    functions!(
        //
        "*" => multiply,
        "note" => note,
        "oscillator" => oscillator,
        "sawtooth" => !,
        "sine" => sine,
        "noise" => !,
        "highPass" => high_pass,
        "lowPass2" => !,
        "highPass2" => !,
        "bandPass2" => !,
        "lowPass4" => !,
        "saturate" => !,
        "rectify" => !,
        "frequency" => !,
        "mix" => !,
        "phase-mod" => !,
        "overtone" => overtone,
    );
    map
}

// =============================================================================================
// Macros
// =============================================================================================

/// Wrap a macro argument with information about its name and source location.
fn macro_arg<'a>(name: &'static str, expr: &'a SExpr) -> EvalResult<&'a SExpr> {
    let label = ValueLabel {
        pos: expr.source_pos(),
        name: Some(name),
        index: 0,
    };
    EvalResult(label, Ok(expr))
}

fn define<'a>(env: &mut Env<'a>, _pos: Span, args: &'a [SExpr]) -> OpResult {
    let (name, value) = match args {
        [name, value] => (macro_arg("name", name), macro_arg("value", value)),
        _ => {
            return Err(OpError::BadNArgs {
                got: args.len(),
                min: 2,
                max: Some(2),
            });
        }
    };
    let mut name = name.and_then(get_symbol);
    match name.value() {
        Some(nameval) => {
            if env.variables.contains_key(nameval) {
                name.1 = error!(
                    env,
                    name.source_pos(),
                    "a variable named {:?} is already defined",
                    nameval
                );
            }
        }
        _ => (),
    };
    let name = name.unwrap(env);
    let value = value.evaluate(env).into_nonvoid().unwrap(env);
    let name = name?;
    env.variables.insert(name, value);
    value?;
    Ok(Value::void())
}

// envelope

// =============================================================================================
// Functions
// =============================================================================================

/// Wrap a function argument with information about its name and source location.
fn func_argn(name: &'static str, index: usize, value: &EvalResult<Value>) -> EvalResult<Value> {
    match value {
        EvalResult(label, value) => {
            let mut label = *label;
            label.name = Some(name);
            label.index = index;
            EvalResult(label, *value)
        }
    }
}

/// Wrap a function argument with information about its name and source location.
fn func_arg(name: &'static str, value: &EvalResult<Value>) -> EvalResult<Value> {
    func_argn(name, 0, value)
}

macro_rules! count_args {
    () => (0usize);
    ($head:ident $($tail:ident)*) => (1usize + count_args!($($tail)*));
}

macro_rules! parse_args {
    ($args:ident, $($name:ident),*) => {
        // TODO: remove the _ in pattern once
        // https://github.com/rust-lang/rust/issues/66295 is fixed in stable.
        let (_, $($name),*) = match $args {
            [$($name),*] => ((), $(func_arg(stringify!($name), $name)),*),
            _ => {
                let n = count_args!($($name)*);
                return Err(OpError::BadNArgs {
                    got: $args.len(),
                    min: n,
                    max: Some(n),
                });
            }
        };
    };
    ($args:ident, $($name:ident),*,) => {
        parse_args!($args, $($name),*)
    };
}

fn new_node(env: &mut Env, pos: Span, units: Units, node: impl Node) -> OpResult {
    Ok(Value(Data::Signal(env.new_node(pos, node)), units))
}

// =============================================================================================
// Parameters
// =============================================================================================

fn note(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, offset);
    let offset = offset
        .into_int()
        .and_then(|i| i32::try_from(i).map_err(|_| unimplemented!()))
        .unwrap(env);
    new_node(env, pos, Units::hertz(1), ops::Note { offset: offset? })
}

// =============================================================================================
// Oscillators and generators
// =============================================================================================

fn oscillator(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, frequency);
    let frequency = frequency.into_signal(Units::hertz(1)).unwrap(env);
    new_node(
        env,
        pos,
        Units::radian(1),
        ops::Oscillator {
            inputs: [frequency?],
        },
    )
}

// sawtooth

fn sine(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, phase);
    let phase = phase.into_signal(Units::radian(1)).unwrap(env);
    new_node(env, pos, Units::volt(1), ops::Sine { inputs: [phase?] })
}

// noise

// =============================================================================================
// Filters
// =============================================================================================

fn high_pass(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, frequency, input);
    let frequency = frequency.into_float(Units::hertz(1)).unwrap(env);
    let input = input.into_signal(Units::volt(1)).unwrap(env);
    new_node(
        env,
        pos,
        Units::volt(1),
        ops::HighPass {
            inputs: [input?],
            frequency: frequency?,
        },
    )
}

// low_pass_2
// high_pass_2
// band_pass_2
// low_pass_4

// =============================================================================================
// Distortion
// =============================================================================================

// saturate
// rectify

// =============================================================================================
// Utilities
// =============================================================================================

fn multiply(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    let (first, rest) = match args.split_first() {
        Some(x) => x,
        None => {
            return Err(OpError::BadNArgs {
                got: args.len(),
                min: 1,
                max: None,
            });
        }
    };
    let mut product = func_argn("arg", 1, first).into_any_signal().unwrap(env);
    for (n, arg) in rest.iter().enumerate() {
        let arg = func_argn("arg", n + 2, arg).into_any_signal().unwrap(env);
        product = match (product, arg) {
            (Ok((xsig, xunits)), Ok((ysig, yunits))) => match xunits.multiply(&yunits) {
                Err(e) => error!(
                    env,
                    pos, "could not multiply {} by {}: {}", xunits, yunits, e
                ),
                Ok(units) => {
                    let sig = env.new_node(
                        pos,
                        ops::Multiply {
                            inputs: [xsig, ysig],
                        },
                    );
                    Ok((sig, units))
                }
            },
            _ => Err(Failed),
        };
    }
    let (sig, units) = product?;
    Ok(Value(Data::Signal(sig), units))
}

// mix
// phase_mod

fn overtone(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, overtone, phase);
    let overtone = overtone
        .into_int()
        .and_then(|i| i32::try_from(i).map_err(|_| unimplemented!()))
        .unwrap(env);
    let phase = phase.into_signal(Units::radian(1)).unwrap(env);
    new_node(
        env,
        pos,
        Units::radian(1),
        ops::ScaleInt {
            inputs: [phase?],
            scale: overtone?,
        },
    )
}
