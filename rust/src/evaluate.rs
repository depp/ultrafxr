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

/// Error for various operations during evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Error {
    Failed,
    BadType { got: Type, expect: Type },
    BadEType { got: EType, expect: EType },
}

impl Error {
    /// True if the error is an evaluation failure, and the diagnostics are
    /// already reported.
    fn is_failed(&self) -> bool {
        match self {
            Error::Failed => true,
            _ => false,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        use Error::*;
        match self {
            Failed => write!(f, "evaluation failed"),
            BadType { got, expect } => write!(f, "type is {}, expected {}", got, expect),
            BadEType { got, expect } => write!(f, "type is {}, expected {}", got, expect),
        }
    }
}

impl From<Failed> for Error {
    fn from(_: Failed) -> Self {
        Error::Failed
    }
}

/// Log an error and return an evaluation failure.
macro_rules! error {
    ($env:expr, $loc:expr, $msg:literal) => {
        {
            $env.error($loc, $msg);
            Err(From::from(Failed))
        }
    };
    ($env:expr, $loc:expr, $($tts:expr),*) => {
        {
            $env.error($loc, format!($($tts),*).as_ref());
            Err(From::from(Failed))
        }
    };
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

    fn bad_type(&self, expect: Type) -> Error {
        Error::BadType {
            got: self.get_type(),
            expect,
        }
    }
}

/// Result of evaluating a form.
type EResult = Result<Value, Failed>;

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

#[derive(Clone, Copy)]
enum Operator {
    Function(fn(&mut Env, Span, &[(Value, Span)]) -> EResult),
    Macro(for<'a> fn(&mut Env<'a>, Span, &'a [SExpr]) -> EResult),
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
    fn evaluate(&mut self, expr: &'a SExpr) -> EResult {
        let pos = expr.source_pos();
        match &expr.content {
            &Content::Symbol(ref name) => match self.variables.get(name.as_ref()) {
                Some(value) => *value,
                None => error!(self, pos, "undefined symbol: {:?}", name),
            },
            &Content::Integer(units, num) => Ok(Value(Data::Int(num), units)),
            &Content::Float(units, num) => Ok(Value(Data::Float(num), units)),
            &Content::List(ref items) => {
                let (op, args) = match items.split_first() {
                    Some(x) => x,
                    None => return error!(self, pos, "cannot evaluate empty list"),
                };
                let op = {
                    let pos = op.source_pos();
                    let name: &str = match &op.content {
                        &Content::Symbol(ref name) => name.as_ref(),
                        _ => return error!(self, pos, "function or macro name must be a symbol"),
                    };
                    match self.operators.get(name) {
                        Some(x) => *x,
                        None => {
                            return error!(self, pos, "undefined function or macro: {:?}", name)
                        }
                    }
                };
                match op {
                    Operator::Function(f) => {
                        let mut values: Vec<(Value, Span)> = Vec::with_capacity(args.len());
                        let mut failed = false;
                        for arg in args.iter() {
                            match self.evaluate(arg) {
                                Err(Failed) => failed = true,
                                Ok(value) => values.push((value, arg.source_pos())),
                            }
                        }
                        if failed {
                            return Err(Failed);
                        }
                        f(self, pos, &values)
                    }
                    Operator::Macro(f) => f(self, pos, args),
                }
            }
        }
    }

    fn error(&mut self, pos: Span, msg: &str) {
        self.has_error = true;
        self.err_handler.handle(pos, msg);
    }

    /// Unwrap the value for an argument.
    fn arg<T>(&mut self, name: &str, pos: Span, value: Result<T, Error>) -> Result<T, Failed> {
        match value {
            Ok(value) => Ok(value),
            Err(Error::Failed) => Err(Failed),
            Err(e) => error!(self, pos, "invalid value for {}: {}", name, e),
        }
    }
}

/// Convert a result to void.
fn get_void(r: EResult) -> Result<(), Error> {
    match r? {
        Value(Data::Void, _) => Ok(()),
        val => Err(val.bad_type(Type::void())),
    }
}

/// Get any non-void value.
fn get_value(r: EResult) -> Result<Value, Error> {
    let val = r?;
    match &val.0 {
        Data::Void => Err(val.bad_type(Type(DataType::NonVoid, None))),
        _ => Ok(val),
    }
}

/// Convert a result to a node with the given units.
fn get_node(units: Units, r: EResult) -> Result<Node, Error> {
    match r? {
        Value(Data::Node(node), vunits) if vunits == units => Ok(node),
        val => Err(val.bad_type(Type(DataType::Node, Some(units)))),
    }
}

/// Get the name of a symbol.
fn get_symbol(expr: &SExpr) -> Result<&str, Error> {
    match &expr.content {
        &Content::Symbol(ref name) => Ok(name),
        _ => Err(Error::BadEType {
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
        match get_void(env.evaluate(form)) {
            Err(e) if !e.is_failed() => {
                env.error(
                    form.source_pos(),
                    format!("bad top-level statement: {}", e).as_ref(),
                );
            }
            _ => (),
        }
    }
    let _node: Node = match get_node(Units::volt(1), env.evaluate(last)) {
        Err(e) => {
            if !e.is_failed() {
                env.error(
                    last.source_pos(),
                    format!("bad program output: {}", e).as_ref(),
                );
            }
            return None;
        }
        Ok(node) => node,
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
        macro_rules! macros {
            ($($f:ident);*;) => {
                $(add(&mut map, stringify!($f), Operator::Macro($f));)*
            };
        }
        macro_rules! functions {
            ($($f:ident);*;) => {
                $(add(&mut map, stringify!($f), Operator::Function($f));)*
            };
        }
        macro_rules! named_functions {
            ($(($name:literal, $f:ident));*;) => {
                $(add(&mut map, $name, Operator::Function($f));)*
            };
        }
        macros!(
            define;
            envelope;
        );
        functions!(
            note;
            oscillator;
            sawtooth;
            sine;
            noise;
            // high_pass;
            // low_pass_2;
            // high_pass_2;
            // band_pass_2;
            // low_pass_4;
            saturate;
            rectify;
            // multiply;
            frequency;
            mix;
            // phase_mod;
            overtone;
        );
        named_functions!(
            ("*", multiply);
            ("phase-mod", phase_mod);
            ("highPass", high_pass);
            ("lowPass2", low_pass_2);
            ("highPass2", high_pass_2);
            ("bandPass2", band_pass_2);
            ("lowPass4", low_pass_4);
        );
        map
    }

    macro_rules! unimplemented {
        ($id:ident) => {
            fn $id<'a>(env: &mut Env<'a>, pos: Span, _args: &'a [SExpr]) -> EResult {
                error!(env, pos, "macro {:?} is unimplemented", stringify!($id))
            }
        };
    }

    fn define<'a>(env: &mut Env<'a>, pos: Span, args: &'a [SExpr]) -> EResult {
        let (name, value) = match args {
            [name, value] => (name, value),
            _ => {
                return error!(env, pos, "define got {} arguments, but needs 2", args.len());
            }
        };
        let namepos = name.source_pos();
        let valuepos = value.source_pos();
        let name = env.arg("name", namepos, get_symbol(name));
        let value = env.evaluate(value);
        let value = env.arg("value", valuepos, get_value(value));
        let name: &'a str = name?;
        if env.variables.contains_key(name) {
            return error!(
                env,
                namepos, "a variable named {:?} is already defined", name
            );
        }
        env.variables.insert(name, value);
        Ok(Value::void())
    }

    unimplemented!(envelope);

    macro_rules! unimplemented {
        ($id:ident) => {
            fn $id(env: &mut Env, pos: Span, _args: &[(Value, Span)]) -> EResult {
                error!(env, pos, "function {:?} is unimplemented", stringify!($id))
            }
        };
    }

    // =============================================================================================
    // Parameters
    // =============================================================================================

    unimplemented!(note);

    // =============================================================================================
    // Oscillators and generators
    // =============================================================================================

    unimplemented!(oscillator);
    unimplemented!(sawtooth);
    unimplemented!(sine);
    unimplemented!(noise);

    // =============================================================================================
    // Filters
    // =============================================================================================

    unimplemented!(high_pass);
    unimplemented!(low_pass_2);
    unimplemented!(high_pass_2);
    unimplemented!(band_pass_2);
    unimplemented!(low_pass_4);

    // =============================================================================================
    // Distortion
    // =============================================================================================

    unimplemented!(saturate);
    unimplemented!(rectify);

    // =============================================================================================
    // Utilities
    // =============================================================================================

    unimplemented!(multiply);
    unimplemented!(frequency);
    unimplemented!(mix);
    unimplemented!(phase_mod);
    unimplemented!(overtone);
}
