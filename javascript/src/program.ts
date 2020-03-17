import { Builder } from './types';
import { Program } from './synthesizer';
import * as operators from './operators';
import * as envelope from './envelope';

export const simpleProgram: Program = {
  bufferCount: 5,
  instantiate(builder: Builder): Float32Array {
    const { buffers } = builder;
    operators.note(builder, buffers[0], 0);
    operators.oscillator(builder, buffers[1], buffers[0]);
    operators.sawtooth(builder, buffers[2], buffers[1]);
    envelope.envelope(builder, buffers[3], 0.1, 0.25);
    operators.multiply(builder, buffers[4], buffers[2], buffers[3]);
    return buffers[4];
  },
};
