use super::environment::*;
use crate::graph::ops::{Envelope, EnvelopeSegment};
use crate::sexpr::{Content, SExpr};
use crate::sourcepos::{HasPos, Span};
use crate::units::Units;

type EnvResult = Result<EnvelopeSegment, OpError>;
type Definition = fn(&mut Env, &[EvalResult<Value>]) -> EnvResult;

fn evaluate<'a>(env: &mut Env<'a>, expr: &'a SExpr) -> Result<EnvelopeSegment, Failed> {
    let pos = expr.source_pos();
    match &expr.content {
        &Content::Symbol(_) => error!(env, pos, "unexpected symbol in envelope"),
        &Content::Integer(_, _) => error!(env, pos, "unexpected number in envelope"),
        &Content::Float(_, _) => error!(env, pos, "unexpected number in envelope"),
        &Content::List(ref items) => {
            let (op, args) = match items.split_first() {
                Some(x) => x,
                None => return error!(env, pos, "cannot evaluate empty list"),
            };
            let name: &str = match &op.content {
                &Content::Symbol(ref name) => name.as_ref(),
                _ => return error!(env, pos, "envelope segment name must be a symbol"),
            };
            let mut values: Vec<EvalResult<Value>> = Vec::with_capacity(args.len());
            for arg in args.iter() {
                values.push(env.evaluate(arg));
            }
            let oppos = op.source_pos();
            let op: Definition = match name {
                "set" => set,
                "lin" => lin,
                "exp" => exp,
                "delay" => delay,
                "gate" => gate,
                "stop" => stop,
                _ => return error!(env, pos, "undefined envelope segment: {:?}", name),
            };
            match op(env, &values) {
                Ok(val) => Ok(val),
                Err(OpError::Failed) => Err(Failed),
                Err(e) => error!(env, oppos, "invalid segment {}: {}", name, e),
            }
        }
    }
}

/// Envelope macro definition.
pub fn envelope<'a>(env: &mut Env<'a>, pos: Span, args: &'a [SExpr]) -> OpResult {
    let mut segments = Vec::with_capacity(args.len());
    let mut failed = false;
    for item in args.iter() {
        match evaluate(env, item) {
            Ok(seg) => segments.push(seg),
            Err(Failed) => failed = true,
        }
    }
    if failed {
        return Err(OpError::Failed);
    }
    segments.shrink_to_fit();
    Ok(Value(
        Data::Signal(env.new_node(pos, Envelope(Box::from(segments)))),
        Units::scalar(),
    ))
}

fn set(env: &mut Env, args: &[EvalResult<Value>]) -> EnvResult {
    parse_args!(args, value);
    let value = value.into_float(Units::scalar()).unwrap(env);
    Ok(EnvelopeSegment::Set(value?))
}

fn lin(env: &mut Env, args: &[EvalResult<Value>]) -> EnvResult {
    parse_args!(args, time, value);
    let time = time.into_float(Units::second(1)).unwrap(env);
    let value = value.into_float(Units::scalar()).unwrap(env);
    Ok(EnvelopeSegment::Lin(time?, value?))
}

fn exp(env: &mut Env, args: &[EvalResult<Value>]) -> EnvResult {
    parse_args!(args, time, value);
    let time = time.into_float(Units::second(1)).unwrap(env);
    let value = value.into_float(Units::scalar()).unwrap(env);
    Ok(EnvelopeSegment::Exp(time?, value?))
}

fn delay(env: &mut Env, args: &[EvalResult<Value>]) -> EnvResult {
    parse_args!(args, time);
    let time = time.into_float(Units::second(1)).unwrap(env);
    Ok(EnvelopeSegment::Delay(time?))
}

fn gate(_env: &mut Env, args: &[EvalResult<Value>]) -> EnvResult {
    parse_args!(args);
    Ok(EnvelopeSegment::Gate)
}

fn stop(_env: &mut Env, args: &[EvalResult<Value>]) -> EnvResult {
    parse_args!(args);
    Ok(EnvelopeSegment::Stop)
}
