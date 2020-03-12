use super::envelope::envelope;
use super::environment::*;
use crate::sexpr::SExpr;
use crate::signal::filter;
use crate::signal::graph::{Node, SignalRef};
use crate::signal::ops;
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
    macro_rules! operator {
        ($kind:ident, $name:literal, !) => {
            add(&mut map, $name, Operator::$kind(None))
        };
        ($kind:ident, $name:literal, $func:ident) => {
            add(&mut map, $name, Operator::$kind(Some($func)))
        };
    }
    macro_rules! operators {
        ($kind:ident, $($name:literal => $val:tt),*,) => {
            $(operator!($kind, $name, $val));*
        };
    }
    operators!(
        Macro,
        "define" => define,
        "envelope" => envelope,
    );
    operators!(
        Function,
        "*" => multiply,
        "note" => note,
        "oscillator" => oscillator,
        "sawtooth" => sawtooth,
        "sine" => sine,
        "noise" => noise,
        "highPass" => high_pass,
        "lowPass2" => low_pass_2,
        "highPass2" => high_pass_2,
        "bandPass2" => band_pass_2,
        "lowPass4" => low_pass_4,
        "saturate" => saturate,
        "rectify" => rectify,
        "frequency" => frequency,
        "mix" => mix,
        "phase-mod" => phase_mod,
        "overtone" => overtone,
    );
    map
}

// =================================================================================================
// Macros
// =================================================================================================

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

// =================================================================================================
// Functions
// =================================================================================================

fn new_node(env: &mut Env, pos: Span, units: Units, node: impl Node + 'static) -> OpResult {
    Ok(Value(Data::Signal(env.new_node(pos, node)), units))
}

// =================================================================================================
// Parameters
// =================================================================================================

fn note(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, offset);
    let offset = offset
        .into_int()
        .and_then(|i| i32::try_from(i).map_err(|_| unimplemented!()))
        .unwrap(env);
    new_node(env, pos, Units::hertz(1), ops::Note { offset: offset? })
}

// =================================================================================================
// Oscillators and generators
// =================================================================================================

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

fn apply_function(
    env: &mut Env,
    pos: Span,
    args: &[EvalResult<Value>],
    function: ops::PointFunction,
    in_units: Units,
    out_units: Units,
) -> OpResult {
    parse_args!(args, input);
    let input = if in_units == Units::radian(1) {
        input.into_phase(env)
    } else {
        input.into_signal(in_units)
    };
    let input = input.unwrap(env)?;
    new_node(env, pos, out_units, ops::ApplyFunction { input, function })
}

fn sine(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    apply_function(
        env,
        pos,
        args,
        ops::PointFunction::Sine,
        Units::radian(1),
        Units::volt(1),
    )
}

fn sawtooth(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    apply_function(
        env,
        pos,
        args,
        ops::PointFunction::Sawtooth,
        Units::radian(1),
        Units::volt(1),
    )
}

fn saturate(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    apply_function(
        env,
        pos,
        args,
        ops::PointFunction::Saturate,
        Units::volt(1),
        Units::volt(1),
    )
}

fn rectify(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    apply_function(
        env,
        pos,
        args,
        ops::PointFunction::Rectify,
        Units::volt(1),
        Units::volt(1),
    )
}

fn noise(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args);
    new_node(env, pos, Units::volt(1), ops::Noise)
}

// =================================================================================================
// Filters
// =================================================================================================

fn high_pass(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, frequency, input);
    let frequency = frequency.into_float(Units::hertz(1)).unwrap(env);
    let input = input.into_signal(Units::volt(1)).unwrap(env);
    new_node(
        env,
        pos,
        Units::volt(1),
        filter::HighPass {
            input: input?,
            frequency: frequency?,
        },
    )
}

fn state_variable(
    env: &mut Env,
    pos: Span,
    args: &[EvalResult<Value>],
    mode: filter::Mode,
) -> OpResult {
    parse_args!(args, input, frequency, q);
    let frequency = frequency.into_signal(Units::hertz(1)).unwrap(env);
    let input = input.into_signal(Units::volt(1)).unwrap(env);
    let q = q.into_float(Units::scalar()).unwrap(env);
    // FIXME: check q >= 0.7
    new_node(
        env,
        pos,
        Units::volt(1),
        filter::StateVariable {
            inputs: [input?, frequency?],
            mode,
            q: q?,
        },
    )
}

fn low_pass_2(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    state_variable(env, pos, args, filter::Mode::LowPass2)
}

fn high_pass_2(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    state_variable(env, pos, args, filter::Mode::HighPass2)
}

fn band_pass_2(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    state_variable(env, pos, args, filter::Mode::BandPass2)
}

fn low_pass_4(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    state_variable(env, pos, args, filter::Mode::LowPass4)
}

// =================================================================================================
// Utilities
// =================================================================================================

fn frequency(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, input);
    let input = input.into_signal(Units::scalar()).unwrap(env);
    new_node(env, pos, Units::hertz(1), ops::Frequency { input: input? })
}

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

fn mix(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    if args.len() & 1 != 0 {
        return error!(
            env,
            pos,
            "got {} arguments, expected an even number",
            args.len()
        );
    }
    let mut output: Result<SignalRef, Failed> = Ok(env.new_node(pos, ops::Zero));
    for (n, chunk) in args.chunks_exact(2).enumerate() {
        let gain = func_argn("gain", n + 1, &chunk[0]).into_gain().unwrap(env);
        let signal = func_argn("signal", n + 1, &chunk[1])
            .into_signal(Units::volt(1))
            .unwrap(env);
        output = match (output, gain, signal) {
            (Ok(xsig), Ok(gain), Ok(ysig)) => Ok(env.new_node(
                pos,
                ops::Mix {
                    inputs: [xsig, ysig],
                    gain,
                },
            )),
            _ => Err(Failed),
        };
    }
    Ok(Value(Data::Signal(output?), Units::volt(1)))
}

fn phase_mod(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    if args.len() & 1 != 1 {
        return error!(
            env,
            pos,
            "got {} arguments, expected an odd number",
            args.len()
        );
    }
    let mut output = func_arg("carrier", &args[0]).into_phase(env).unwrap(env);
    for (n, chunk) in args[1..].chunks_exact(2).enumerate() {
        let gain = func_argn("gain", n + 1, &chunk[0]).into_gain().unwrap(env);
        let modulator = func_argn("modulator", n + 1, &chunk[1])
            .into_signal(Units::volt(1))
            .unwrap(env);
        output = match (output, gain, modulator) {
            (Ok(xsig), Ok(gain), Ok(ysig)) => Ok(env.new_node(
                pos,
                ops::Mix {
                    inputs: [xsig, ysig],
                    gain,
                },
            )),
            _ => Err(Failed),
        };
    }
    Ok(Value(Data::Signal(output?), Units::radian(1)))
}

fn overtone(env: &mut Env, pos: Span, args: &[EvalResult<Value>]) -> OpResult {
    parse_args!(args, overtone, phase);
    let overtone = overtone
        .into_int()
        .and_then(|i| i32::try_from(i).map_err(|_| unimplemented!()))
        .unwrap(env);
    let phase = phase.into_phase(env).unwrap(env);
    new_node(
        env,
        pos,
        Units::radian(1),
        ops::ScaleInt {
            input: phase?,
            scale: overtone?,
        },
    )
}
