use std::time::Duration;

/// if not mco
#[cfg(feature = "runtime_thread")]
pub type TcpListener = std::net::TcpListener;
#[cfg(feature = "runtime_thread")]
pub type TcpStream = std::net::TcpStream;
#[cfg(feature = "runtime_thread")]
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
#[cfg(feature = "runtime_thread")]
pub type Sender<T> = crossbeam::channel::Sender<T>;
#[cfg(feature = "runtime_thread")]
pub type JoinHandle<T> = std::thread::JoinHandle<T>;
#[cfg(feature = "runtime_thread")]
pub type Mutex<T> = std::sync::Mutex<T>;
#[cfg(feature = "runtime_thread")]
pub type WaitGroup = crossbeam_utils::sync::WaitGroup;

#[cfg(feature = "runtime_thread")]
pub fn chan<T>() -> (Sender<T>, Receiver<T>) {
    crossbeam::channel::unbounded()
}

#[cfg(feature = "runtime_thread")]
pub fn sleep(d: Duration) {
    std::thread::sleep(d)
}

#[cfg(feature = "runtime_thread")]
pub fn spawn<F>(f: F) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    std::thread::spawn(f)
}

#[cfg(feature = "runtime_thread")]
pub fn spawn_stack_size<F>(f: F, stack_size:usize) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    std::thread::spawn(f)
}