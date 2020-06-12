# Operators

A program in UltraFXR is made by connecting basic operators into a graph. Here are the defined operators:

## Filtering

- 2-pole filter: Operates in low-pass, high-pass, band-pass, or notch modes. Takes signal, cutoff frequency, and Q inputs.

## Envelopes

> TODO: Figure out attack / decay curves.

- AD envelope: Rises and then falls in response to trigger.
- AR envelope: Rises while gate is active, falls when gate is off.
- ADSR envelope: Traditional ADSR. Rises and then falls to sustain level while gate is active, falls when gate is off. Can be configured as retriggerable or not retriggerable.

## Oscillators

- Oscillator: Generates phase in range -0.5..+0.5 from frequency input.
- Sawtooth: Generates sawtooth waveform from phase input.
- Sine: Generates sine wave from phase input. Configurable approximation - paraboloid, 3rd order, 5th order, etc.
- Triangle: Generate triangle wave from phase input.
- Pulse: Generate pulse wave from phase input and duty cycle input.
- Uniform noise: Generates uniform noise.
- Gaussian noise: Generates gaussian noise.
- Pink noise: Generates pink noise.

## Wave Shaping

- Absolute Value: Computes absolute value of input.
- Min: Outputs smaller of two inputs.
- Max: Outputs larger of two inputs.
- Clamp: Clamps input to between minimum and maximum value.
- Soft Clip: Piecewise polynomial saturation. Configurable order.
- Hyperbolic Soft Clip: Output is hyperbolic tangent of input.
- Square: Compute square of input.
- Square root: Compute square root of input.
- Exponential: Compute the exponential of an input.
- Logarithm: Compute the logarithm of an input.

## Utility

- Mix: Add two signals, applying a fixed gain to one of them.
- Add: Add two signals.
- Multiply: Multiply two signals.
- Multiply Integer: Multiply a signal by a fixed integer.
- Constant: Generate constant signal.

## TODO

- EQ, beyond simple filtering
- Delay
- Reverb
- Tools for more synthesis techniques: physical modeling, modal, etc.
