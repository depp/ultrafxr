import { Builder, noteA4, frequencyA4 } from './types';

/**
 * Create note frequency node.
 * @param builder Synthesizer builder context.
 * @param output Output, frequency of note in Hz.
 * @param offset Transposition value added to input MIDI note.
 */
export function note(
  builder: Builder,
  output: Float32Array,
  offset: number,
): void {
  const { note } = builder.parameters;
  const frequency = frequencyA4 * Math.pow(2, (note + offset - noteA4) / 12);
  builder.addRenderer(function renderNote(): void {
    output.fill(frequency);
  });
}

/**
 * Create oscillator phase node.
 * @param builder Synthesizer builder context.
 * @param output Output, oscillator phase in -0.5..+0.5.
 * @param frequency Input frequnecy of note in Hz.
 */
export function oscillator(
  builder: Builder,
  output: Float32Array,
  frequency: Float32Array,
): void {
  const { sampleRate } = builder.parameters;
  let phase = 0;
  const multiplier = 1 / sampleRate;
  builder.addRenderer(function renderOscillator(): void {
    for (let i = 0; i < output.length; i++) {
      output[i] = phase;
      phase += multiplier * frequency[i];
      if (phase > 0.5) {
        phase -= 0.5;
      }
    }
  });
}

/**
 * Create sawtooth waveform node.
 * @param builder Synthesizer builder context.
 * @param output Output, sawtooth waveform in -1.0..+1.0.
 * @param phase Input phase, nominally in -0.5..+0.5.
 */
export function sawtooth(
  builder: Builder,
  output: Float32Array,
  phase: Float32Array,
): void {
  builder.addRenderer(function renderSawtooth(): void {
    for (let i = 0; i < output.length; i++) {
      let x = phase[i];
      output[i] = 2 * (x - Math.round(x));
    }
  });
}

/**
 * Create multiplication node.
 * @param builder Synthesizer builder context.
 * @param output Output, equal to inputs multiplied.
 * @param xinput First input.
 * @param yinput Second input.
 */
export function multiply(
  builder: Builder,
  output: Float32Array,
  xinput: Float32Array,
  yinput: Float32Array,
): void {
  builder.addRenderer(function renderMultiply(): void {
    for (let i = 0; i < output.length; i++) {
      output[i] = xinput[i] * yinput[i];
    }
  });
}
