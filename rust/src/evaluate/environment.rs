use crate::error::ErrorHandler;
use crate::sexpr::{Content, SExpr, Type as EType};
use crate::signal::graph::{Graph, Node, SignalRef};
use crate::sourcepos::{HasPos, Span};
use crate::units::Units;
use std::collections::hash_map::{HashMap, RandomState};
use std::convert::From;
use std::fmt::{Display, Formatter, Result as FResult};

/// Error for evaluation failure. The error diagnostics are reported elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Failed;

/// Log an error, returning void.
macro_rules! log_error {
    ($env:expr, $loc:expr, $msg:literal) => {
        $env.error($loc, $msg);
    };
    ($env:expr, $loc:expr, $($tts:expr),*) => {
        $env.error($loc, format!($($tts),*).as_ref());
    };
}

/// Log an error and return an evaluation failure.
macro_rules! error {
    ($env:expr, $loc:expr, $($tts:expr),*) => {{
        log_error!($env, $loc, $($tts),*);
        Err(From::from($crate::evaluate::environment::Failed))
    }};
}

/// Error for interpreting a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueError {
    Failed,
    BadType { got: Type, expect: Type },
    BadEType { got: EType, expect: EType },
    BadGain { got: Type },
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        use ValueError::*;
        match self {
            Failed => write!(f, "evaluation failed"),
            BadType { got, expect } => write!(f, "type is {}, expected {}", got, expect),
            BadEType { got, expect } => write!(f, "type is {}, expected {}", got, expect),
            BadGain { got } => write!(f, "type is {}, expected gain (dB or scalar constant)", got),
        }
    }
}

impl From<Failed> for ValueError {
    fn from(_: Failed) -> Self {
        ValueError::Failed
    }
}

/// Error for various operations during evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpError {
    Failed,
    BadNArgs {
        got: usize,
        min: usize,
        max: Option<usize>,
    },
}

impl Display for OpError {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        use OpError::*;
        match self {
            Failed => write!(f, "evaluation failed"),
            BadNArgs { got, min, max } => match max {
                Some(max) => {
                    if min < max {
                        write!(f, "got {} args, expected {}-{}", got, min, max)
                    } else {
                        write!(f, "got {} args, expected {}", got, min)
                    }
                }
                None => write!(f, "got {} args, expected at least {}", got, min),
            },
        }
    }
}

impl From<Failed> for OpError {
    fn from(_: Failed) -> Self {
        OpError::Failed
    }
}

/// The data within a value, without units.
#[derive(Debug, Clone, Copy)]
pub enum Data {
    Int(i64),
    Float(f64),
    Signal(SignalRef),
    Void,
}

impl Data {
    fn data_type(&self) -> DataType {
        match self {
            Data::Int(_) => DataType::Int,
            Data::Float(_) => DataType::Float,
            Data::Signal(_) => DataType::Signal,
            Data::Void => DataType::Void,
        }
    }
}

/// A value from evaluating an expression successfully.
#[derive(Debug, Clone, Copy)]
pub struct Value(pub Data, pub Units);

/// Convert from decibels to a scalar ratio.
fn db_to_ratio(db: f64) -> f64 {
    (db * (10.0f64.ln() / 20.0)).exp()
}

impl Value {
    pub fn void() -> Self {
        Value(Data::Void, Units::scalar())
    }

    fn get_type(&self) -> Type {
        Type(self.0.data_type(), Some(self.1))
    }

    fn bad_type(&self, expect: Type) -> ValueError {
        ValueError::BadType {
            got: self.get_type(),
            expect,
        }
    }

    fn into_void(self) -> Result<(), ValueError> {
        match self {
            Value(Data::Void, _) => Ok(()),
            _ => Err(self.bad_type(Type::void())),
        }
    }

    fn into_nonvoid(self) -> Result<Value, ValueError> {
        match self {
            Value(Data::Void, _) => Err(self.bad_type(Type(DataType::NonVoid, None))),
            val => Ok(val),
        }
    }

    fn into_int(self) -> Result<i64, ValueError> {
        match self {
            Value(Data::Int(num), units) if units.is_scalar() => Ok(num),
            val => Err(val.bad_type(Type(DataType::Int, Some(Units::scalar())))),
        }
    }

    fn into_float(self, units: Units) -> Result<f64, ValueError> {
        match self {
            Value(Data::Float(num), vunits) if units == vunits => Ok(num),
            Value(Data::Int(num), vunits) if units == vunits => Ok(num as f64),
            val => Err(val.bad_type(Type(DataType::Float, Some(units)))),
        }
    }

    fn into_gain(self) -> Result<f64, ValueError> {
        let err = ValueError::BadGain {
            got: self.get_type(),
        };
        let (num, units) = match self {
            Value(Data::Float(num), units) => (num, units),
            Value(Data::Int(num), units) => (num as f64, units),
            _ => return Err(err),
        };
        if units == Units::decibel(1) {
            Ok(db_to_ratio(num))
        } else if units == Units::scalar() {
            Ok(num)
        } else {
            Err(err)
        }
    }

    fn into_any_signal(self) -> Result<(SignalRef, Units), ValueError> {
        match self {
            Value(Data::Signal(sig), units) => Ok((sig, units)),
            val => Err(val.bad_type(Type(DataType::Signal, None))),
        }
    }

    fn into_signal(self, units: Units) -> Result<SignalRef, ValueError> {
        match self {
            Value(Data::Signal(sig), vunits) if vunits == units => Ok(sig),
            val => Err(val.bad_type(Type(DataType::Signal, Some(units)))),
        }
    }
}

/// Result of evaluating function or macro body.
pub type OpResult = Result<Value, OpError>;

/// A type of data within a value, or void.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Void,
    Signal,
    Int,
    Float,
    NonVoid,
}

/// A type of a value with its units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Type(pub DataType, pub Option<Units>);

impl Type {
    fn void() -> Self {
        Type(DataType::Void, None)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.write_str(match self.0 {
            DataType::Void => "void",
            DataType::Signal => "signal",
            DataType::Int => "int",
            DataType::Float => "float",
            DataType::NonVoid => "non-void",
        })?;
        match self.1 {
            Some(u) => write!(f, "({})", u),
            None => Ok(()),
        }
    }
}

/// Identifying information about a value so we can generate better error
/// messages for it.
#[derive(Debug, Clone, Copy)]
pub struct ValueLabel {
    pub pos: Span,
    pub name: Option<&'static str>,
    pub index: usize,
}

/// Result of evaluating a subexpression.
#[derive(Debug, Clone)]
pub struct EvalResult<T>(pub ValueLabel, pub Result<T, ValueError>);

impl<T> EvalResult<T> {
    pub fn and_then<U, F>(self, op: F) -> EvalResult<U>
    where
        F: FnOnce(T) -> Result<U, ValueError>,
    {
        match self {
            EvalResult(label, v) => EvalResult(label, v.and_then(op)),
        }
    }

    pub fn unwrap(self, env: &mut Env) -> Result<T, Failed> {
        let EvalResult(label, value) = self;
        match value {
            Ok(value) => Ok(value),
            Err(ValueError::Failed) => Err(Failed),
            Err(e) => {
                let msg = match label.name {
                    Some(name) => format!("invalid value for {}: {}", name, e),
                    None => format!("invalid value: {}", e),
                };
                env.error(label.pos, msg.as_str());
                Err(Failed)
            }
        }
    }
}

impl<T> EvalResult<T>
where
    T: Copy,
{
    pub fn value(&self) -> Option<T> {
        match self {
            EvalResult(_, Ok(v)) => Some(*v),
            _ => None,
        }
    }
}

impl<T> HasPos for EvalResult<T> {
    fn source_pos(&self) -> Span {
        match self {
            EvalResult(label, _) => label.pos,
        }
    }
}

impl EvalResult<Value> {
    pub fn into_void(self) -> EvalResult<()> {
        self.and_then(Value::into_void)
    }

    pub fn into_nonvoid(self) -> EvalResult<Value> {
        self.and_then(Value::into_nonvoid)
    }

    pub fn into_int(self) -> EvalResult<i64> {
        self.and_then(Value::into_int)
    }

    pub fn into_float(self, units: Units) -> EvalResult<f64> {
        self.and_then(|v| v.into_float(units))
    }

    pub fn into_gain(self) -> EvalResult<f64> {
        self.and_then(Value::into_gain)
    }

    pub fn into_any_signal(self) -> EvalResult<(SignalRef, Units)> {
        self.and_then(Value::into_any_signal)
    }

    pub fn into_signal(self, units: Units) -> EvalResult<SignalRef> {
        self.and_then(|v| v.into_signal(units))
    }
}

impl<'a> EvalResult<&'a SExpr> {
    pub fn evaluate(self, env: &mut Env<'a>) -> EvalResult<Value> {
        let EvalResult(label, value) = self;
        let value = match value {
            Ok(form) => match env.evaluate_impl(form) {
                Ok(v) => Ok(v),
                Err(Failed) => Err(ValueError::Failed),
            },
            Err(e) => Err(e),
        };
        EvalResult(label, value)
    }
}

#[derive(Clone, Copy)]
pub enum Operator {
    Function(Option<fn(&mut Env, Span, &[EvalResult<Value>]) -> OpResult>),
    Macro(Option<for<'a> fn(&mut Env<'a>, Span, &'a [SExpr]) -> OpResult>),
}

/// An environment for evaluating s-expressions.
pub struct Env<'a> {
    has_error: bool,
    err_handler: &'a mut dyn ErrorHandler,
    pub variables: HashMap<&'a str, Result<Value, Failed>, RandomState>,
    operators: HashMap<&'a str, Operator, RandomState>,
    graph: Graph,
    #[allow(dead_code)]
    tail_length: Option<f64>,
}

impl<'a> Env<'a> {
    /// Create a new environment with the given functions defined.
    pub fn new(
        err_handler: &'a mut dyn ErrorHandler,
        operators: HashMap<&'a str, Operator, RandomState>,
    ) -> Self {
        Env {
            has_error: false,
            err_handler,
            variables: HashMap::new(),
            operators,
            graph: Graph::new(),
            tail_length: None,
        }
    }

    /// Evaluate an s-expression.
    pub fn evaluate(&mut self, expr: &'a SExpr) -> EvalResult<Value> {
        let label = ValueLabel {
            pos: expr.source_pos(),
            name: None,
            index: 0,
        };
        let value = match self.evaluate_impl(expr) {
            Ok(value) => Ok(value),
            Err(Failed) => Err(ValueError::Failed),
        };
        EvalResult(label, value)
    }

    fn evaluate_impl(&mut self, expr: &'a SExpr) -> Result<Value, Failed> {
        let pos = expr.source_pos();
        match &expr.content {
            &Content::Symbol(ref name) => match self.variables.get(name.as_ref()) {
                Some(&value) => value,
                None => error!(self, pos, "undefined symbol: {:?}", name),
            },
            &Content::Integer(units, num) => Ok(Value(Data::Int(num), units)),
            &Content::Float(units, num) => Ok(Value(Data::Float(num), units)),
            &Content::List(ref items) => {
                let (op, args) = match items.split_first() {
                    Some(x) => x,
                    None => return error!(self, pos, "cannot evaluate empty list"),
                };
                let name: &str = match &op.content {
                    &Content::Symbol(ref name) => name.as_ref(),
                    _ => return error!(self, pos, "function or macro name must be a symbol"),
                };
                let oppos = op.source_pos();
                let op = match self.operators.get(name) {
                    Some(x) => *x,
                    None => return error!(self, oppos, "undefined function or macro: {:?}", name),
                };
                let r = match op {
                    Operator::Function(f) => {
                        if f.is_none() {
                            log_error!(self, oppos, "unimplemented function: {}", name);
                        }
                        let mut values: Vec<EvalResult<Value>> = Vec::with_capacity(args.len());
                        for arg in args.iter() {
                            values.push(self.evaluate(arg));
                        }
                        match f {
                            Some(f) => f(self, pos, &values),
                            None => Err(OpError::Failed),
                        }
                    }
                    Operator::Macro(f) => match f {
                        Some(f) => f(self, pos, args),
                        None => error!(self, oppos, "unimplemented macro: {}", name),
                    },
                };
                match r {
                    Ok(val) => Ok(val),
                    Err(OpError::Failed) => Err(Failed),
                    Err(e) => error!(self, oppos, "invalid call to {}: {}", name, e),
                }
            }
        }
    }

    /// Log an error message.
    pub fn error(&mut self, pos: Span, msg: &str) {
        self.has_error = true;
        self.err_handler.handle(pos, msg);
    }

    /// Add a new audio processing node to the graph.
    pub fn new_node(&mut self, pos: Span, node: impl Node) -> SignalRef {
        let _ = pos;
        self.graph.add(Box::new(node))
    }

    /// Discard the environment and return the created graph.
    pub fn into_graph(self) -> Option<Graph> {
        if self.has_error {
            None
        } else {
            Some(self.graph)
        }
    }
}

/// Get the name of a symbol.
pub fn get_symbol(expr: &SExpr) -> Result<&str, ValueError> {
    match &expr.content {
        &Content::Symbol(ref name) => Ok(name),
        _ => Err(ValueError::BadEType {
            got: expr.get_type(),
            expect: EType::Symbol,
        }),
    }
}

/// Wrap a function argument with information about its name and source location.
pub fn func_argn(name: &'static str, index: usize, value: &EvalResult<Value>) -> EvalResult<Value> {
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
pub fn func_arg(name: &'static str, value: &EvalResult<Value>) -> EvalResult<Value> {
    func_argn(name, 0, value)
}

macro_rules! count_args {
    () => (0usize);
    ($head:ident $($tail:ident)*) => (1usize + count_args!($($tail)*));
}

macro_rules! parse_args {
    ($args:ident) => {
        if !$args.is_empty() {
            return Err(OpError::BadNArgs {
                got: $args.len(),
                min: 0,
                max: Some(0),
            });
        }
    };
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
