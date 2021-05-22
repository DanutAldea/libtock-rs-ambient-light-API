/// Unit tests can use `ExpectedSyscall` to alter `fake::Kernel`'s behavior for
/// a particular system call. An example use case is error injection: unit tests
/// can add a `ExpectedSyscall` to the fake kernel's queue to insert errors in
/// order to test error handling code.
#[derive(Clone, Copy, Debug)]
pub enum ExpectedSyscall {
    // -------------------------------------------------------------------------
    // Yield
    // -------------------------------------------------------------------------
    YieldNoWait {
        /// If not `None`, `yield-no-wait` will set the return value to the
        /// specified value. If `None`, `yield-no-wait` will set the return
        /// value based on whether or not an upcall was run.
        override_return: Option<libtock_platform::YieldNoWaitReturn>,
    },

    YieldWait {
        /// If true, yield_wait will skip executing a upcall.
        skip_upcall: bool,
    },

    // TODO: Add Subscribe.

    // -------------------------------------------------------------------------
    // Command
    // -------------------------------------------------------------------------
    Command {
        // Matched values: the command must give the specified driver_id,
        // command_id, argument0, and argument1 values.
        driver_id: u32,
        command_id: u32,
        argument0: u32,
        argument1: u32,

        // If not None, the output of the driver will be replaced with the given
        // return value.
        override_return: Option<libtock_platform::CommandReturn>,
    },
    // TODO: Add Allow.
    // TODO: Add Memop.
    // TODO: Add Exit.
}

impl ExpectedSyscall {
    // Panics with a message describing that the named system call was called
    // instead of the expected system call. Used by fake::Kernel to report
    // incorrect system calls.
    pub(crate) fn panic_wrong_call(&self, called: &str) -> ! {
        // TODO: Implement Display for ExpectedSyscall and replace {:?} with {}
        panic!(
            "Expected system call {:?}, but {} was called instead.",
            self, called
        );
    }
}
