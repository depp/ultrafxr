import { EventType, createSynthesizer, simpleProgram } from './src/index';
import { WaveBuilder } from './src/wave';

import * as fs from 'fs';
import * as path from 'path';

/**
 * Generate a sound and return chunks of a wave file.
 */
function genWave(): ArrayBuffer[] {
  const sampleRate = 48000;
  const synth = createSynthesizer(simpleProgram, {
    sampleRate,
    bufferSize: 2048,
    note: 60,
  });
  synth.sendEvent(EventType.Gate, 0, 1);
  synth.sendEvent(EventType.Gate, sampleRate / 2, 0);
  synth.stop(sampleRate * 10);
  const { output } = synth;
  const wave = new WaveBuilder(sampleRate, 1);
  let count = 0;
  while (true) {
    const amt = synth.render();
    if (amt != null) {
      wave.write(output.subarray(0, amt));
      break;
    }
    wave.write(output);
    count += output.length;
    if (count > sampleRate * 60 * 2) {
      throw new Error('synthesizer produced too much output, aborting');
    }
  }
  return wave.chunks();
}

(async function main(): Promise<void> {
  try {
    const chunks = genWave();
    const outPath = path.join(__dirname, 'out.wav');
    const file = await fs.promises.open(outPath, 'w');
    for (const chunk of chunks) {
      const r = await file.write(Buffer.from(chunk));
    }
    await file.sync();
    await file.close();
  } catch (e) {
    console.error(e);
    process.exit(1);
  }
})();
