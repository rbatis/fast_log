use crossbeam_utils::sync::WaitGroup;

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
    pub fn wait_exit(self) {
        self.inner.wait();
    }
    /// exit and wait log empty
    pub fn exit_wait(self) {
        crate::fast_log::exit();
        self.inner.wait();
    }
}