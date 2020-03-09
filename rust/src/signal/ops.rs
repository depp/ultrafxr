use super::graph::{Node, NodeResult, SignalRef};
use super::program::{Function, Parameters, State};
use std::error;
use std::f32;
use std::fmt::{Display, Formatter, Result as FResult};

/// Unimplemented operator error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unimplemented(pub &'static str);

impl Display for Unimplemented {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "unimplemented node type: {}", self.0)
    }
}

impl error::Error for Unimplemented {}

fn unimplemented(name: &'static str) -> NodeResult {
    Err(Box::from(Unimplemented(name)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FilterMode {
    LowPass2,
    HighPass2,
    BandPass2,
    LowPass4,
}

macro_rules! count_inputs {
    () => (0usize);
    ($head:ident $($tail:ident)*) => (1usize + count_inputs!($($tail)*));
}

macro_rules! op {
    ($name:ident, [], []) => {
        #[derive(Debug)]
        pub struct $name;
        impl Node for $name {
            fn inputs(&self) -> &[SignalRef] {
                &[]
            }
            fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
                unimplemented(stringify!($name))
            }
        }
    };
    ($name:ident, [], [$($pname:ident: $ptype:ty),*]) => {
        #[derive(Debug)]
        pub struct $name {
            $(pub $pname: $ptype),*
        }
        impl Node for $name {
            fn inputs(&self) -> &[SignalRef] {
                &[]
            }
            fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
                unimplemented(stringify!($name))
            }
        }
    };
    ($name:ident, [$($input:ident),*], [$($pname:ident: $ptype:ty),*]) => {
        #[derive(Debug)]
        pub struct $name {
            pub inputs: [SignalRef; count_inputs!($($input)*)],
            $(pub $pname: $ptype),*
        }
        impl Node for $name {
            fn inputs(&self) -> &[SignalRef] {
                &self.inputs[..]
            }
            fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
                unimplemented(stringify!($name))
            }
        }
    };
    ($name:ident, [$($input:ident),*], [$($pname:ident: $ptype:ty),*],) => {
        op!($name, [$($input),*], [$($pname: $ptype),*]);
    };
    ($name:ident, [$($input:ident),*],) => {
        op!($name, [$($input),*], []);
    };
    ($name:ident, [$($input:ident),*]) => {
        op!($name, [$($input),*], []);
    };
}

// =================================================================================================
// Oscillators and generators
// =================================================================================================

/// Generate phase from frequency.
#[derive(Debug)]
pub struct Oscillator {
    pub inputs: [SignalRef; 1],
}

impl Node for Oscillator {
    fn inputs(&self) -> &[SignalRef] {
        &self.inputs[..]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::from(OscillatorF {
            scale: 1.0 / 48000.0,
            phase: 0.0,
        }))
    }
}

#[derive(Debug)]
struct OscillatorF {
    scale: f32,
    phase: f32,
}

impl Function for OscillatorF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &State) {
        let frequency = &inputs[0][0..output.len()];
        let scale = self.scale;
        let mut phase = self.phase;
        for (output, &frequency) in output.iter_mut().zip(frequency.iter()) {
            *output = phase;
            phase += frequency * scale;
        }
        self.phase = phase;
    }
}

// =================================================================================================

/// Generate sine waveform from phase.
#[derive(Debug)]
pub struct Sine {
    pub inputs: [SignalRef; 1],
}

impl Node for Sine {
    fn inputs(&self) -> &[SignalRef] {
        &self.inputs[..]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::from(SineF))
    }
}

#[derive(Debug)]
struct SineF;

impl Function for SineF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &State) {
        let phase = &inputs[0][0..output.len()];
        for (output, &phase) in output.iter_mut().zip(phase.iter()) {
            *output = (phase * (2.0 * f32::consts::PI)).sin();
        }
    }
}

// =================================================================================================

op!(Sawtooth, [phase]);
op!(Noise, []);

// Filters
op!(HighPass, [input], [frequency: f64]);
op!(
    StateVariableFilter,
    [input, frequency],
    [mode: FilterMode, q: f64],
);

// Distortion
op!(Saturate, [input]);
op!(Rectify, [input]);

// Envelopes
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeSegment {
    Set(f64),
    Lin(f64, f64),
    Exp(f64, f64),
    Delay(f64),
    Gate,
    Stop,
}

#[derive(Debug)]
pub struct Envelope(pub Box<[EnvelopeSegment]>);

impl Node for Envelope {
    fn inputs(&self) -> &[SignalRef] {
        &[]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        unimplemented("Envelope")
    }
}

// Utilities
op!(Multiply, [x, y]);
op!(Constant, [], [value: f64]);
op!(Frequency, [input]);
op!(Mix, [base, input], [gain: f64]);
op!(Zero, []);
op!(ScaleInt, [input], [scale: i32]);

/*
// Parameter references
op!(Parameter, 0, 0, 1); // -> deref and derefcopy
op!(Note, 1, 0, 1);
 */

// =================================================================================================

/// Generate input note frequency.
#[derive(Debug)]
pub struct Note {
    /// Offset to apply to input note, in semitones.
    pub offset: i32,
}

impl Node for Note {
    fn inputs(&self) -> &[SignalRef] {
        &[]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::from(NoteF {
            offset: self.offset,
        }))
    }
}

#[derive(Debug)]
struct NoteF {
    offset: i32,
}

impl Function for NoteF {
    fn render(&mut self, output: &mut [f32], _inputs: &[&[f32]], state: &State) {
        let frequency =
            440.0 * 2.0f32.powf((state.note() + (self.offset - 69) as f32) * (1.0 / 12.0));
        for x in output.iter_mut() {
            *x = frequency;
        }
    }
}
