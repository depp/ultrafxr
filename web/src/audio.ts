/**
 * The active audio context, or null if it hasn't been created yet.
 */
let globalAudioCtx: AudioContext | null = null;

/**
 * Start playback of an audio buffer.
 * @param audioCtx The active audio context.
 * @param buffer The buffer to play.
 */
function playBuffer(audioCtx: AudioContext, buffer: AudioBuffer): void {
  const source = audioCtx.createBufferSource();
  source.buffer = buffer;
  source.connect(audioCtx.destination);
  source.start();
}

/**
 * Initialize the audio context and make it ready, if it is not already ready.
 * This should be called from an event handler in order to avoid autoplay
 * blocking.
 */
function startAudioContext(): AudioContext | null {
  let audioCtx = globalAudioCtx;
  if (audioCtx != null) {
    return audioCtx;
  }
  try {
    // Recent versions of Safari still don't have AudioContext.
    audioCtx = new ((window as any).AudioContext ||
      (window as any).webkitAudioContext)() as AudioContext;
  } catch (e) {
    console.error(e);
    return null;
  }
  globalAudioCtx = audioCtx;
  return audioCtx;
}

/**
 * Start playback of a test tone: a triangle wave at 440 Hz.
 */
export function playSound(): void {
  const audioCtx = startAudioContext();
  if (audioCtx == null) {
    return;
  }
  const { sampleRate } = audioCtx;
  const frequency = 440;
  const delta = frequency / sampleRate;
  const buffer = audioCtx.createBuffer(1, sampleRate, sampleRate);
  const data = buffer.getChannelData(0);
  let phase = 0;
  for (let i = 0; i < sampleRate; i++) {
    data[i] = 4 * Math.abs(phase) - 2;
    phase += delta;
    if (phase > 0.5) {
      phase -= 1;
    }
  }
  playBuffer(audioCtx, buffer);
}
