#[cfg(feature = "runtime_thread")]
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
#[cfg(feature = "runtime_thread")]
pub type Sender<T> = crossbeam::channel::Sender<T>;
#[cfg(feature = "runtime_thread")]
pub type SendError<T> = crossbeam_channel::SendError<T>;
#[cfg(feature = "runtime_thread")]
pub type RecvError = crossbeam_channel::RecvError;
#[cfg(feature = "runtime_thread")]
pub type JoinHandle<T> = std::thread::JoinHandle<T>;
#[cfg(feature = "runtime_thread")]
pub type WaitGroup = crossbeam_utils::sync::WaitGroup;

#[cfg(feature = "runtime_thread")]
pub fn chan<T>(len: Option<usize>) -> (Sender<T>, Receiver<T>) {
    match len {
        None => crossbeam::channel::unbounded(),
        Some(len) => crossbeam::channel::bounded(len),
    }
}

#[cfg(feature = "runtime_thread")]
pub fn spawn<F>(f: F) -> JoinHandle<()>
where
    F: FnOnce() + Send + 'static,
{
    std::thread::spawn(f)
}

#[cfg(feature = "runtime_thread")]
pub fn spawn_stack_size<F>(f: F, stack_size: usize) -> JoinHandle<()>
where
    F: FnOnce() + Send + 'static,
{
    std::thread::spawn(f)
}
