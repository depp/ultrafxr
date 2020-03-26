let audioCtx = null;

function getAudioContext() {
  if (audioCtx != null) {
    return audioCtx;
  }
  let ctor = window.AudioContext;
  if (ctor == null) {
    ctor = window.webkitAudioContext;
  }
  if (ctor == null) {
    return null;
  }
  const ctx = new ctor();
  audioCtx = ctx;
  return ctx;
}

function makeBuffer(ctx) {
  const { sampleRate } = ctx;
  const length = (sampleRate * 0.25) | 0;
  const buffer = ctx.createBuffer(1, length, sampleRate);
  const arr = buffer.getChannelData(0);
  const frequency = 440 * Math.pow(2, -9 / 12);
  const slope = (2 * frequency) / sampleRate;
  let value = 0.0;
  for (let i = 0; i < arr.length; i++) {
    arr[i] = value;
    value += slope;
    if (value > 1.0) {
      value -= 2.0;
    }
  }
  return buffer;
}

function playBuffer(ctx, buffer) {
  const source = ctx.createBufferSource();
  source.buffer = buffer;
  source.connect(ctx.destination);
  source.start();
}

export function playSound() {
  const ctx = getAudioContext();
  if (ctx == null) {
    return;
  }
  const buffer = makeBuffer(ctx);
  playBuffer(ctx, buffer);
}
