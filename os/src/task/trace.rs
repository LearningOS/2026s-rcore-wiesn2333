#[derive(Copy, Clone)]
/// A struct for tracking system call counts
pub struct Trace {
    /// Number of write system calls
    pub write: usize,
    /// Number of exit system calls
    pub exit: usize,
    /// Number of yield system calls
    pub yield_: usize,
    /// Number of get_time system calls
    pub get_time: usize,
    /// Number of trace system calls
    pub trace: usize,
}

impl Trace {
    /// Creates a new `Trace` instance with all counts initialized to zero.
    pub fn new() -> Self {
        Self {
            write: 0,
            exit: 0,
            yield_: 0,
            get_time: 0,
            trace: 0,
        }
    }
}
