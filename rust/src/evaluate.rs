use crate::error::ErrorHandler;
use crate::sexpr::{Content, SExpr, Type as EType};
use crate::sourcepos::{HasPos, Span};
use crate::units::Units;
use std::collections::hash_map::{HashMap, RandomState};
use std::convert::From;
use std::fmt::{Display, Formatter, Result as FResult};

/// Error for evaluation failure. The error diagnostics are reported elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Failed;

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
        Err(From::from(Failed))
    }};
}

/// Error for interpreting a value.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ValueError {
    Failed,
    BadType { got: Type, expect: Type },
    BadEType { got: EType, expect: EType },
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        use ValueError::*;
        match self {
            Failed => write!(f, "evaluation failed"),
            BadType { got, expect } => write!(f, "type is {}, expected {}", got, expect),
            BadEType { got, expect } => write!(f, "type is {}, expected {}", got, expect),
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
enum OpError {
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

#[derive(Debug, Clone, Copy)]
struct Node;

/// The data within a value, without units.
#[derive(Debug, Clone, Copy)]
enum Data {
    Int(i64),
    Float(f64),
    #[allow(dead_code)]
    Node(Node),
    Void,
}

impl Data {
    fn data_type(&self) -> DataType {
        match self {
            Data::Int(_) => DataType::Int,
            Data::Float(_) => DataType::Float,
            Data::Node(_) => DataType::Node,
            Data::Void => DataType::Void,
        }
    }
}

/// A value from evaluating an expression successfully.
#[derive(Debug, Clone, Copy)]
struct Value(Data, Units);

impl Value {
    fn void() -> Self {
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

    fn into_node(self, units: Units) -> Result<Node, ValueError> {
        match self {
            Value(Data::Node(node), vunits) if vunits == units => Ok(node),
            _ => Err(self.bad_type(Type(DataType::Node, Some(units)))),
        }
    }
}

/// Result of evaluating function or macro body.
type OpResult = Result<Value, OpError>;

/// A type of data within a value, or void.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataType {
    Void,
    Node,
    Int,
    Float,
    NonVoid,
}

/// A type of a value with its units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Type(DataType, Option<Units>);

impl Type {
    fn void() -> Self {
        Type(DataType::Void, None)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.write_str(match self.0 {
            DataType::Void => "void",
            DataType::Node => "buffer",
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
struct ValueLabel {
    pos: Span,
    name: Option<&'static str>,
    index: usize,
}

/// Result of evaluating a subexpression.
#[derive(Debug, Clone)]
struct EvalResult<T>(ValueLabel, Result<T, ValueError>);

impl<T> EvalResult<T> {
    fn and_then<U, F>(self, op: F) -> EvalResult<U>
    where
        F: FnOnce(T) -> Result<U, ValueError>,
    {
        match self {
            EvalResult(label, v) => EvalResult(label, v.and_then(op)),
        }
    }

    fn unwrap(self, env: &mut Env) -> Result<T, Failed> {
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
    fn value(&self) -> Option<T> {
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
    fn into_void(self) -> EvalResult<()> {
        self.and_then(Value::into_void)
    }

    fn into_nonvoid(self) -> EvalResult<Value> {
        self.and_then(Value::into_nonvoid)
    }

    fn into_node(self, units: Units) -> EvalResult<Node> {
        self.and_then(|v| v.into_node(units))
    }
}

impl<'a> EvalResult<&'a SExpr> {
    fn evaluate(self, env: &mut Env<'a>) -> EvalResult<Value> {
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

/// Wrap a macro argument with information about its name and source location.
fn macro_arg<'a>(name: &'static str, expr: &'a SExpr) -> EvalResult<&'a SExpr> {
    let label = ValueLabel {
        pos: expr.source_pos(),
        name: Some(name),
        index: 0,
    };
    EvalResult(label, Ok(expr))
}

#[derive(Clone, Copy)]
enum Operator {
    Function(Option<fn(&mut Env, Span, &[EvalResult<Value>]) -> OpResult>),
    Macro(Option<for<'a> fn(&mut Env<'a>, Span, &'a [SExpr]) -> OpResult>),
}

struct Env<'a> {
    has_error: bool,
    err_handler: &'a mut dyn ErrorHandler,
    variables: HashMap<&'a str, Result<Value, Failed>, RandomState>,
    operators: HashMap<&'a str, Operator, RandomState>,
    #[allow(dead_code)]
    tail_length: Option<f64>,
}

impl<'a> Env<'a> {
    fn evaluate(&mut self, expr: &'a SExpr) -> EvalResult<Value> {
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
                    Err(e) => error!(self, pos, "invalid call to {:?}: {}", name, e),
                }
            }
        }
    }

    fn error(&mut self, pos: Span, msg: &str) {
        self.has_error = true;
        self.err_handler.handle(pos, msg);
    }
}

/// Get the name of a symbol.
fn get_symbol(expr: &SExpr) -> Result<&str, ValueError> {
    match &expr.content {
        &Content::Symbol(ref name) => Ok(name),
        _ => Err(ValueError::BadEType {
            got: expr.get_type(),
            expect: EType::Symbol,
        }),
    }
}

/// Evaluate an audio synthesis program.
pub fn evaluate_program(err_handler: &mut dyn ErrorHandler, program: &[SExpr]) -> Option<()> {
    // Break program into the leading forms and the last form. The last form is
    // considered to be the output, and must produce a value.
    let (last, first) = match program.split_last() {
        None => {
            err_handler.handle(Span::none(), "empty program");
            return None;
        }
        Some(x) => x,
    };
    let mut env = Env {
        has_error: false,
        err_handler,
        variables: HashMap::new(),
        operators: ops::operators(),
        tail_length: None,
    };
    for form in first.iter() {
        match env.evaluate(form).into_void() {
            EvalResult(_, Ok(())) => (),
            EvalResult(label, Err(e)) => match e {
                ValueError::Failed => (),
                _ => log_error!(env, label.pos, "invalid top-level statement: {}", e),
            },
        }
    }
    let _node: Node = match env.evaluate(last).into_node(Units::volt(1)) {
        EvalResult(_, Ok(node)) => node,
        EvalResult(label, Err(e)) => {
            match e {
                ValueError::Failed => (),
                _ => log_error!(env, label.pos, "invalid program body: {}", e),
            }
            return None;
        }
    };
    if env.has_error {
        return None;
    }
    Some(())
}

// =================================================================================================

mod ops {
    use super::*;

    pub(super) fn operators() -> HashMap<&'static str, Operator, RandomState> {
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
        const UNIMPL_FUNCS: &'static [&'static str] = &[
            //
            "*",
            "note",
            "oscillator",
            "sawtooth",
            "sine",
            "noise",
            "highPass",
            "lowPass2",
            "highPass2",
            "bandPass2",
            "lowPass4",
            "saturate",
            "rectify",
            "multiply",
            "frequency",
            "mix",
            "phase-mod",
            "overtone",
        ];
        for &name in UNIMPL_MACROS.iter() {
            add(&mut map, name, Operator::Macro(None));
        }
        for &name in UNIMPL_FUNCS.iter() {
            add(&mut map, name, Operator::Function(None));
        }
        macro_rules! macros {
            ($($f:ident);*;) => {
                $(add(&mut map, stringify!($f), Operator::Macro(Some($f)));)*
            };
        }
        #[allow(unused_macros)]
        macro_rules! functions {
            ($($f:ident);*;) => {
                $(add(&mut map, stringify!($f), Operator::Function(Some($f)));)*
            };
        }
        #[allow(unused_macros)]
        macro_rules! named_functions {
            ($(($name:literal, $f:ident));*;) => {
                $(add(&mut map, $name, Operator::Function($f));)*
            };
        }
        macros!(
            define;
        );
        map
    }

    // =============================================================================================
    // Macros
    // =============================================================================================

    fn define<'a>(env: &mut Env<'a>, _pos: Span, args: &'a [SExpr]) -> OpResult {
        let (name, value) = match args {
            [name, value] => (macro_arg("name", name), macro_arg("value", value)),
            _ => {
                return Err(OpError::BadNArgs {
                    got: args.len(),
                    min: 2,
                    max: Some(2),
                })
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
    // Parameters
    // =============================================================================================

    // note

    // =============================================================================================
    // Oscillators and generators
    // =============================================================================================

    // oscillator
    // sawtooth
    // sine
    // noise

    // =============================================================================================
    // Filters
    // =============================================================================================

    // high_pass
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

    // multiply
    // frequency
    // mix
    // phase_mod
    // overtone
}
