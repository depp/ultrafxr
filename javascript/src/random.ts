/**
 * Random number generator.
 *
 * We use a custom PRNG rather than Math.random() to make it possible to
 * generate the same levels on different browsers, since different browsers use
 * different PRNGs.
 */
export class Random {
  // This uses the Xorshift32 algorithm by George Marsaglia.

  /** The state, which can be used to manually seed. */
  state: number;

  constructor(seed?: number) {
    this.state = seed || 1;
  }

  /**
   * Return a pseudorandom integer in the range 1..2^32-1.
   */
  next(): number {
    let { state } = this;
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (this.state = state >>> 0);
  }

  /**
   * Return a pseudorandom floating-point number in the half-open range from 0
   * to 1, including 0 but excluding 1.
   */
  nextFloat(): number {
    return this.next() * (1 / 4294967296.0);
  }
}
