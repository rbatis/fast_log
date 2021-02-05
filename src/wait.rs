use crossbeam_utils::sync::WaitGroup;

/// In the case of multithreading, you need to call clone and drop at the end func
#[derive(Clone, Debug)]
pub struct FastLogWaitGroup {
    pub inner: WaitGroup,
}

impl FastLogWaitGroup {
    pub fn new()->Self{
        Self{
            inner: WaitGroup::new()
        }
    }
    /// wait call fast_log::exit();
    pub fn wait(self) {
        self.inner.wait();
    }
    /// exit and wait log empty
    pub fn exit_and_wait(self) {
        crate::fast_log::exit();
        self.inner.wait();
    }

    ///send exit msg
    pub fn exit(self) {
        crate::fast_log::exit();
    }
}