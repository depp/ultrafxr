// See: https://www.pcg-random.org/download.html

/// State for a random number generator.
#[derive(Clone)]
pub struct Rand {
    state: u64,
    inc: u64,
}

impl Rand {
    /// Create a random number generator with the default seed.
    pub fn with_default_seed() -> Self {
        Rand {
            state: 0x853c49e6748fea9b,
            inc: 0xda3e39cb94b95bdb,
        }
    }

    /// Create a random number generator with the given seed.
    pub fn with_seed(seed: u64, seq: u64) -> Self {
        let mut r = Rand {
            state: 0,
            inc: (seq << 1) | 1,
        };
        r.next();
        r.state = r.state.overflowing_add(seed).0;
        r.next();
        r
    }

    /// Return the next number in the sequence.
    pub fn next(&mut self) -> u32 {
        let state = self.state;
        self.state = state
            .overflowing_mul(6364136223846793005)
            .0
            .overflowing_add(self.inc)
            .0;
        ((((state >> 18) ^ state) >> 27) as u32).rotate_right((state >> 59) as u32)
    }

    /// Return the next number in the sequence, scaled to 0.0-1.0.
    pub fn next_float(&mut self) -> f32 {
        (self.next() as f32) * (1.0 / 4294967296.0)
    }
}
