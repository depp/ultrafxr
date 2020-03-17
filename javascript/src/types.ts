/**
 * Parameters for instantiating a synthesizer.
 */
export interface Params {
  /** The size of the audio rendering buffer, in samples. */
  readonly bufferSize: number;
  /** The audio system sample rate in Hz. */
  readonly sampleRate: number;
  /** The MIDI value of the note to play. */
  readonly note: number;
}

/**
 * Types of events.
 */
export const enum EventType {
  /**
   * Gate (trigger) event. Sent with a value of 1 when the note starts, and a
   * value of 0 when the note ends.
   */
  Gate = 'gate',
}

/**
 * A synthesizer event. Events are dispatched and resolved before audio is
 * rendered.
 */
export interface Event {
  /** The type of event. */
  eventType: EventType;

  /**
   * The time when the event triggers. This is measured relative to the start
   * of the next buffer to be rendered.
   */
  time: number;

  /**
   * The value associated with the event. This has a different meaning for
   * different event types.
   */
  value: number;
}

export interface RenderContext {
  /**
   * Stop the synthesizer.
   *
   * @param time The time at which to stop the synthesizer, measured in samples
   * from the start of the current buffer.
   */
  stop(time: number): void;
}

export interface EventContext extends RenderContext {
  /**
   * Send an event. The event will not be dispatched and resolved immediately,
   * but will be queued. Events are resolved in order by timestamp and then by
   * the order in which they were sent.
   *
   * Events may not be sent to the past.
   *
   * @param event The event type.
   * @param time The event time, measured in samples relative to the next buffer
   * start.
   * @param value A numeric value associated with the event.
   */
  sendEvent(eventType: EventType, time: number, value: number): void;
}

/**
 * A handler for synthesizer events.
 */
export type EventHandler = (ctx: EventContext, event: Event) => void;

/**
 * An audio renderer callback.
 */
export type Renderer = (ctx: RenderContext) => void;

/**
 * Builder for instantiating signal processing nodes.
 */
export interface Builder {
  readonly parameters: Params;
  readonly buffers: readonly Float32Array[];
  addEventListener(eventType: EventType, handler: EventHandler): void;
  addRenderer(renderer: Renderer): void;
}

/** The MIDI note value of A4. */
export const noteA4 = 69;

/** The frequency that A4 sounds at. */
export const frequencyA4 = 440;
