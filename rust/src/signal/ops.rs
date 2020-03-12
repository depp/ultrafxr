use super::graph::{Node, NodeResult, SignalRef};
use super::program::{Function, Parameters, State};
use std::error;
use std::f32;
use std::fmt::{Display, Formatter, Result as FResult};
use std::slice::from_ref;

/// Unimplemented operator error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unimplemented(pub &'static str);

impl Display for Unimplemented {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "unimplemented node type: {}", self.0)
    }
}

impl error::Error for Unimplemented {}

#[allow(dead_code)]
fn unimplemented(name: &'static str) -> NodeResult {
    Err(Box::new(Unimplemented(name)))
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
        Ok(Box::new(OscillatorF {
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
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        let frequency = &inputs[0][0..output.len()];
        let scale = self.scale;
        let mut phase = self.phase;
        for (output, &frequency) in output.iter_mut().zip(frequency.iter()) {
            *output = phase;
            phase += frequency * scale;
            if phase > 1.0 {
                phase -= 1.0;
            }
        }
        self.phase = phase;
    }
}

// =================================================================================================

/// Types of waveforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointFunction {
    Sine,
    Sawtooth,
    Saturate,
    Rectify,
}

/// Apply a function to the waveform.
#[derive(Debug)]
pub struct ApplyFunction {
    pub input: SignalRef,
    pub function: PointFunction,
}

impl Node for ApplyFunction {
    fn inputs(&self) -> &[SignalRef] {
        from_ref(&self.input)
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(ApplyFunctionF(self.function)))
    }
}

#[derive(Debug)]
struct ApplyFunctionF(PointFunction);

impl Function for ApplyFunctionF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        let items = output.iter_mut().zip(inputs[0].iter());
        use PointFunction::*;
        match self.0 {
            Sine => {
                for (output, &phase) in items {
                    *output = (phase * (2.0 * f32::consts::PI)).sin();
                }
            }
            Sawtooth => {
                for (output, &phase) in items {
                    *output = phase * 2.0 - 1.0;
                }
            }
            Saturate => {
                for (y, &x) in items {
                    *y = x.tanh();
                }
            }
            Rectify => {
                for (y, &x) in items {
                    *y = x.abs();
                }
            }
        }
    }
}

// =================================================================================================

/// Generate uniform noise at the full sample rate.
#[derive(Debug)]
pub struct Noise;

impl Node for Noise {
    fn inputs(&self) -> &[SignalRef] {
        &[]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(NoiseF))
    }
}

#[derive(Debug)]
struct NoiseF;

impl Function for NoiseF {
    fn render(&mut self, output: &mut [f32], _inputs: &[&[f32]], state: &mut State) {
        let rand = state.rand();
        for output in output.iter_mut() {
            *output = rand.next_float() * 2.0 - 1.0
        }
    }
}

// =================================================================================================

/// Multiply two inputs.
#[derive(Debug)]
pub struct Multiply {
    pub inputs: [SignalRef; 2],
}

impl Node for Multiply {
    fn inputs(&self) -> &[SignalRef] {
        &self.inputs[..]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(MultiplyF))
    }
}

#[derive(Debug)]
struct MultiplyF;

impl Function for MultiplyF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        let inputx = inputs[0];
        let inputy = inputs[1];
        for (output, (&x, &y)) in output.iter_mut().zip(inputx.iter().zip(inputy.iter())) {
            *output = x * y;
        }
    }
}

// =================================================================================================

/// Multiply an input by a constant gain and add it to the base signal.
#[derive(Debug)]
pub struct Mix {
    /// (base, input) => base + gain * input
    pub inputs: [SignalRef; 2],
    pub gain: f64,
}

impl Node for Mix {
    fn inputs(&self) -> &[SignalRef] {
        &self.inputs[..]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(MixF {
            gain: self.gain as f32,
        }))
    }
}

#[derive(Debug)]
struct MixF {
    gain: f32,
}

impl Function for MixF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        for (output, (&base, &input)) in output
            .iter_mut()
            .zip(inputs[0].iter().zip(inputs[1].iter()))
        {
            *output = base + self.gain * input;
        }
    }
}

// =================================================================================================

/// Convert numbers from -1..+1 to 20..20000, exponentially.
#[derive(Debug)]
pub struct Frequency {
    pub input: SignalRef,
}

impl Node for Frequency {
    fn inputs(&self) -> &[SignalRef] {
        from_ref(&self.input)
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(FrequencyF))
    }
}

#[derive(Debug)]
struct FrequencyF;

impl Function for FrequencyF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        for (y, &x) in output.iter_mut().zip(inputs[0].iter()) {
            *y = 630.0 * 32.0f32.powf(x);
        }
    }
}

// =================================================================================================

/// Create a zero buffer.
#[derive(Debug)]
pub struct Zero;

impl Node for Zero {
    fn inputs(&self) -> &[SignalRef] {
        &[]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(ZeroF))
    }
}

#[derive(Debug)]
struct ZeroF;

impl Function for ZeroF {
    fn render(&mut self, output: &mut [f32], _inputs: &[&[f32]], _state: &mut State) {
        for output in output.iter_mut() {
            *output = 0.0;
        }
    }
}

// =================================================================================================

/// Scale input by an integer.
#[derive(Debug)]
pub struct ScaleInt {
    pub input: SignalRef,
    pub scale: i32,
}

impl Node for ScaleInt {
    fn inputs(&self) -> &[SignalRef] {
        from_ref(&self.input)
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(ScaleIntF {
            scale: self.scale as f32,
        }))
    }
}

#[derive(Debug)]
struct ScaleIntF {
    scale: f32,
}

impl Function for ScaleIntF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        for (output, &input) in output.iter_mut().zip(inputs[0].iter()) {
            *output = input * self.scale
        }
    }
}

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
        Ok(Box::new(NoteF {
            offset: self.offset,
        }))
    }
}

#[derive(Debug)]
struct NoteF {
    offset: i32,
}

impl Function for NoteF {
    fn render(&mut self, output: &mut [f32], _inputs: &[&[f32]], state: &mut State) {
        let frequency =
            440.0 * 2.0f32.powf((state.note() + (self.offset - 69) as f32) * (1.0 / 12.0));
        for x in output.iter_mut() {
            *x = frequency;
        }
    }
}

// =================================================================================================

/// Generate a constant value.
#[derive(Debug)]
pub struct Constant {
    pub value: f32,
}

impl Node for Constant {
    fn inputs(&self) -> &[SignalRef] {
        &[]
    }
    fn instantiate(&self, _parameters: &Parameters) -> NodeResult {
        Ok(Box::new(ConstantF { value: self.value }))
    }
}

#[derive(Debug)]
struct ConstantF {
    value: f32,
}

impl Function for ConstantF {
    fn render(&mut self, output: &mut [f32], _inputs: &[&[f32]], _state: &mut State) {
        for output in output.iter_mut() {
            *output = self.value;
        }
    }
}
