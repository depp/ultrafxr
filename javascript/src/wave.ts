import { Random } from './random';

const chunkSize = 16 * 1024;

/**
 * Class which builds a WAVE file in memory.
 */
export class WaveBuilder {
  private readonly sampleRate: number;
  private readonly channelCount: number;
  private readonly rand: Random = new Random(0x01234567);
  private readonly chunkList: ArrayBuffer[] = [];
  private chunk: Int16Array | null = new Int16Array(chunkSize);
  private chunkPos: number = 22; // Skip header of 44 bytes.

  constructor(sampleRate: number, channelCount: number) {
    this.sampleRate = sampleRate;
    this.channelCount = channelCount;
  }

  /**
   * Write an array of floating-point samples to the WAVE file.
   * @param data Floating-point samples in the range 0-1.
   */
  write(data: Float32Array): void {
    let pos = 0;
    let { chunk, chunkPos } = this;
    while (pos < data.length) {
      if (chunk == null) {
        chunk = new Int16Array(chunkSize);
        chunkPos = 0;
      }
      let count = Math.min(chunk.length - chunkPos, data.length - pos);
      for (let i = 0; i < count; i++) {
        let sample = Math.floor(data[pos + i] * 0x8000 + this.rand.nextFloat());
        if (sample > 0x7fff) {
          sample = 0x7fff;
        } else if (sample < -0x8000) {
          sample = -0x8000;
        }
        chunk[chunkPos + i] = sample;
      }
      pos += count;
      chunkPos += count;
      if (chunkPos < chunk.length) {
        break;
      }
      this.chunkList.push(chunk.buffer);
      chunk = null;
      chunkPos = 0;
    }
    this.chunk = chunk;
    this.chunkPos = chunkPos;
  }

  /**
   * Get a list of all chunks of data in the WAVE file.
   */
  chunks(): ArrayBuffer[] {
    const chunks = [...this.chunkList];
    if (this.chunk != null) {
      chunks.push(this.chunk.buffer.slice(0, this.chunkPos * 2));
    }
    let fileSizeBytes = 0;
    for (const chunk of chunks) {
      fileSizeBytes += chunk.byteLength;
    }
    const sampleSizeBytes = 2;
    const frameSizeBytes = this.channelCount * sampleSizeBytes;

    // Write WAVE header.
    const view = new DataView(chunks[0], 0, 44);
    view.setUint32(0, 0x46464952, true); // RIFF
    view.setUint32(4, fileSizeBytes - 8, true);
    view.setUint32(8, 0x45564157, true); // WAVE

    // Start of fmt chunk.
    view.setUint32(12, 0x20746d66, true); // fmt
    view.setUint32(16, 16, true);
    view.setUint16(20, 1, true); // format: 1 = PCM
    view.setUint16(22, 1, true); // channel count
    view.setUint32(24, this.sampleRate, true);
    view.setUint32(28, this.sampleRate * frameSizeBytes, true);
    view.setUint16(32, frameSizeBytes, true);
    view.setUint16(34, sampleSizeBytes * 8, true);

    // Start of data chunk.
    view.setUint32(36, 0x61746164, true); // data
    view.setUint32(40, fileSizeBytes - 44, true);

    return chunks;
  }
}
