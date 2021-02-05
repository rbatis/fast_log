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
    pub fn wait(self) {
        crate::fast_log::exit();
        self.inner.wait();
    }
}