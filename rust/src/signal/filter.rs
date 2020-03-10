use super::graph::{Node, NodeResult, SignalRef};
use super::program::{Function, Parameters, State};
use std::f64;
use std::slice::from_ref;

// =================================================================================================

/// The mode for a state-variable filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    LowPass2,
    HighPass2,
    BandPass2,
    LowPass4,
}

/// A state-variable filter with a control input for frequency.
#[derive(Debug)]
pub struct StateVariable {
    /// (input, frequency)
    pub inputs: [SignalRef; 2],
    pub mode: Mode,
    pub q: f64,
}

impl Node for StateVariable {
    fn inputs(&self) -> &[SignalRef] {
        &self.inputs[..]
    }
    fn instantiate(&self, parameters: &Parameters) -> NodeResult {
        let q = match self.mode {
            Mode::LowPass4 => (self.q * 0.5f64.sqrt()).sqrt(),
            _ => self.q,
        };
        Ok(Box::new(StateVariableF {
            stage: [SVF([0.0, 0.0]), SVF([0.0, 0.0])],
            temp: {
                let mut temp = Vec::<f32>::new();
                temp.resize(parameters.buffer_size, 0.0);
                Box::from(temp)
            },
            scale: ((2.0 * f64::consts::PI) / parameters.sample_rate) as f32,
            mode: self.mode,
            invq: (1.0 / q) as f32,
        }))
    }
}

#[derive(Debug)]
struct StateVariableF {
    stage: [SVF; 2],
    temp: Box<[f32]>,
    scale: f32,
    mode: Mode,
    invq: f32,
}

impl Function for StateVariableF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        let input = inputs[0];
        let frequency = inputs[1];
        let temp = &mut self.temp[..];
        for (y, &x) in temp.iter_mut().zip(frequency.iter()) {
            // FIXME: should be 10k
            *y = (self.scale * (x * 0.5).min(20000.0)).sin();
        }
        match self.mode {
            Mode::LowPass2 => self.stage[0].render_lp(output, input, temp, self.invq),
            Mode::HighPass2 => self.stage[0].render_hp(output, input, temp, self.invq),
            Mode::BandPass2 => self.stage[0].render_bp(output, input, temp, self.invq),
            Mode::LowPass4 => {
                self.stage[0].render_lp(output, input, temp, self.invq);
                self.stage[1].render_lp(output, input, temp, self.invq);
            }
        }
    }
}

// =================================================================================================

/// A two-pole high pass filter with Q=0.707 and fixed frequency.
#[derive(Debug)]
pub struct HighPass {
    pub input: SignalRef,
    pub frequency: f64,
}

impl Node for HighPass {
    fn inputs(&self) -> &[SignalRef] {
        from_ref(&self.input)
    }
    fn instantiate(&self, parameters: &Parameters) -> NodeResult {
        Ok(Box::new(HighPassF {
            svf: SVF([0.0, 0.0]),
            frequency: {
                let mut temp = Vec::<f32>::new();
                // FIXME: should be 10k
                let value = (2.0 * f64::consts::PI * (self.frequency * 0.5).min(20000.0)
                    / parameters.sample_rate)
                    .sin() as f32;
                temp.resize(parameters.buffer_size, value);
                Box::from(temp)
            },
        }))
    }
}

#[derive(Debug)]
struct HighPassF {
    svf: SVF,
    frequency: Box<[f32]>,
}

impl Function for HighPassF {
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], _state: &mut State) {
        self.svf
            .render_hp(output, inputs[0], &self.frequency[..], 2.0f32.sqrt());
    }
}

// =================================================================================================

/// Mode for a state variable filter.
#[derive(Debug)]
enum SVFMode {
    LowPass,
    HighPass,
    BandPass,
}

/// State for a state variable filter.
#[derive(Debug)]
struct SVF([f32; 2]);

impl SVF {
    fn render(
        &mut self,
        output: &mut [f32],
        input: &[f32],
        frequency: &[f32],
        invq: f32,
        mode: SVFMode,
    ) {
        let mut state = self.0;
        for (output, (&x, &f)) in output.iter_mut().zip(input.iter().zip(frequency.iter())) {
            // We oversample the filter, running it twice with a corner
            // frequency scaled by 1/2. Without oversampling, the filter stops
            // working well at high frequencies.
            let [a, b] = state;
            let b = b + f * a;
            let c = x - b - invq * a;
            let a = a + f * c;
            let b = b + f * a;
            let c = x - b - invq * a;
            let a = a + f * c;
            *output = match mode {
                SVFMode::LowPass => b,
                SVFMode::HighPass => c,
                SVFMode::BandPass => a,
            };
            state = [a, b];
        }
        self.0 = state;
    }

    fn render_lp(&mut self, output: &mut [f32], input: &[f32], frequency: &[f32], invq: f32) {
        self.render(output, input, frequency, invq, SVFMode::LowPass);
    }

    fn render_hp(&mut self, output: &mut [f32], input: &[f32], frequency: &[f32], invq: f32) {
        self.render(output, input, frequency, invq, SVFMode::HighPass);
    }

    fn render_bp(&mut self, output: &mut [f32], input: &[f32], frequency: &[f32], invq: f32) {
        self.render(output, input, frequency, invq, SVFMode::BandPass);
    }
}
