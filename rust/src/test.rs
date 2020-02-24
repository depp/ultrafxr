use std::error::Error;
use std::fmt;

/// Wrapper for bytestrings that pretty prints them.
pub struct Str<'a>(pub &'a [u8]);

impl<'a> fmt::Display for Str<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Str(b) = self;
        match std::str::from_utf8(b) {
            Ok(s) => fmt::Debug::fmt(s, f),
            _ => fmt::Debug::fmt(b, f),
        }
    }
}

/// Error type for failed tests.
#[derive(Debug)]
pub struct TestFailure;

impl fmt::Display for TestFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test failed")
    }
}

impl Error for TestFailure {}

/// An object which tracks the number of successful and failed tests.
pub struct Tests {
    success: u32,
    failure: u32,
}

impl Tests {
    /// Create a new test result tracker.
    pub fn new() -> Tests {
        Tests {
            success: 0,
            failure: 0,
        }
    }

    /// Record the result of a subtest.
    pub fn add(&mut self, success: bool) -> bool {
        if success {
            self.success += 1;
        } else {
            self.failure += 1;
        }
        success
    }

    /// Finish a group of tests.
    pub fn done(self) -> Result<(), TestFailure> {
        if self.failure > 0 {
            eprintln!(
                "Error: {} of {} tests failed.",
                self.failure,
                self.success + self.failure
            );
            Err(TestFailure)
        } else {
            Ok(())
        }
    }
}
