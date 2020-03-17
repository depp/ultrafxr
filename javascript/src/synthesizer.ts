import {
  Params,
  EventType,
  Event,
  EventContext,
  Builder,
  EventHandler,
  Renderer,
} from './types';

export interface Synthesizer extends EventContext {
  /** The output buffer. This buffer is filled by calling render(). */
  output: Float32Array;

  /** Render the next buffer of output. */
  render(): number | null;
}

/**
 * Create an array of buffers.
 * @param size The number of samples in each buffer.
 * @param count The number of buffers to create.
 */
function createBuffers(size: number, count: number): Float32Array[] {
  const backBuffer = new Float32Array(size * count);
  const buffers: Float32Array[] = [];
  for (let i = 0; i < count; i++) {
    buffers.push(backBuffer.subarray(i * size, (i + 1) * size));
  }
  return buffers;
}

export interface Program {
  bufferCount: number;
  instantiate(builder: Builder): Float32Array;
}

/**
 * Create a synthesizer which runs the given program.
 */
export function createSynthesizer(
  program: Program,
  parameters: Params,
): Synthesizer {
  // Program instantiation.
  const { bufferSize } = parameters;
  const listeners = new Map<EventType, EventHandler[]>();
  const renderers: Renderer[] = [];

  function addEventListener(eventType: EventType, handler: EventHandler): void {
    let list = listeners.get(eventType);
    if (list == null) {
      list = [];
      listeners.set(eventType, list);
    }
    list.push(handler);
  }

  function addRenderer(renderer: Renderer): void {
    renderers.push(renderer);
  }

  const { bufferCount } = program;
  const buffers = createBuffers(bufferSize, bufferCount);
  const builder: Builder = {
    parameters,
    buffers,
    addEventListener,
    addRenderer,
  };
  const output = program.instantiate(builder);

  // Program context.
  let stopPos: number | null = null;
  let currentTime: number = 0;
  const events: Event[] = [];

  function stop(time: number): void {
    if (time < 0) {
      throw new Error('cannot stop in the past');
    }
    if (stopPos == null) {
      stopPos = time;
    } else {
      stopPos = Math.min(stopPos, time);
    }
  }

  function sendEvent(eventType: EventType, time: number, value: number): void {
    time = time | 0;
    if (time < currentTime) {
      throw new Error('cannot send event to the past');
    }
    const event: Event = { eventType, time, value };
    for (let i = 0; i < events.length; i++) {
      if (events[i].time > time) {
        events.splice(i, 0, event);
        return;
      }
    }
    events.push(event);
  }

  const ctx: EventContext = { stop, sendEvent };

  function render(): number | null {
    while (events.length != 0) {
      const event = events[0];
      const { time } = event;
      if (time > bufferSize) {
        break;
      }
      events.shift();
      const list = listeners.get(event.eventType);
      if (list != null) {
        currentTime = time;
        for (const handler of list) {
          handler(ctx, event);
        }
      }
    }
    for (const renderer of renderers) {
      renderer(ctx);
    }
    for (const event of events) {
      event.time -= bufferSize;
    }
    currentTime = 0;
    if (stopPos != null) {
      if (stopPos <= bufferSize) {
        return stopPos;
      }
      stopPos -= bufferSize;
    }
    return null;
  }

  return {
    stop,
    sendEvent,
    output,
    render,
  };
}
