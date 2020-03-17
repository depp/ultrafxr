import {
  RenderContext,
  Builder,
  EventType,
  Event,
  EventContext,
} from './types';

/**
 * Create an AR envelope. The envelope will go from 0 to 1 during the attack
 * time when the gate starts, and holde at 1. When the gate is released, the
 * envelope will go from its current value (possibly interrupting the attack)
 * to 0 at a slope determined by the release time.
 *
 * @param builder Synthesizer builder context.
 * @param output Output envelope value.
 * @param attack Time for attack to go to full value, in seconds.
 * @param release Time for release to go from full value, in seconds.
 */
export function envelope(
  builder: Builder,
  output: Float32Array,
  attack: number,
  release: number,
): void {
  const { sampleRate } = builder.parameters;
  const attackDelta = 1 / (attack * sampleRate);
  const releaseDelta = -1 / (release * sampleRate);
  const enum State {
    Zero,
    Attack,
    Sustain,
    Release,
  }
  let state = State.Zero;
  let outputPos = 0;
  let value = 0;
  function renderPart(ctx: RenderContext, start: number, end: number): void {
    let pos = start;
    while (pos < end) {
      switch (state) {
        case State.Zero:
          output.fill(0, pos, end);
          return;
        case State.Attack:
          for (; pos < end; pos++) {
            value += attackDelta;
            if (value >= 1.0) {
              value = 1.0;
              state = State.Sustain;
              break;
            }
            output[pos] = value;
          }
          break;
        case State.Sustain:
          output.fill(1, pos, end);
          return;
        case State.Release:
          for (; pos < end; pos++) {
            value += releaseDelta;
            if (value <= 0.0) {
              value = 0.0;
              state = State.Zero;
              ctx.stop(pos);
              break;
            }
            output[pos] = value;
          }
          break;
      }
    }
  }
  builder.addEventListener(EventType.Gate, function gateEnvelope(
    ctx: EventContext,
    event: Event,
  ) {
    const { time, value } = event;
    renderPart(ctx, outputPos, time);
    outputPos = time;
    if (value >= 0.5) {
      state = State.Attack;
    } else {
      state = State.Release;
    }
  });
  builder.addRenderer(function renderEnvelope(ctx: RenderContext): void {
    renderPart(ctx, outputPos, output.length);
    outputPos = 0;
  });
}
