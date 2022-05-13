use crate::WaitGroup;

/// In the case of multithreading, you need to call clone and drop at the end func
#[derive(Clone, Debug)]
pub struct FastLogWaitGroup {
    pub inner: WaitGroup,
}

impl FastLogWaitGroup {
    pub fn new() -> Self {
        Self {
            inner: WaitGroup::new(),
        }
    }
    /// wait call fast_log::exit();
    pub fn do_wait(self) {
        self.inner.wait();
    }
    /// wait call fast_log::exit();
    pub fn wait(self) {
        crate::fast_log::exit();
        self.inner.wait();
    }
    ///send exit msg
    pub fn exit(self) {
        crate::fast_log::exit();
    }
}

impl Default for FastLogWaitGroup {
    fn default() -> Self {
        Self::new()
    }
}
