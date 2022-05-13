use std::time::Duration;

/// if not mco
#[cfg(not(feature = "mco"))]
pub type TcpListener = std::net::TcpListener;
#[cfg(not(feature = "mco"))]
pub type TcpStream = std::net::TcpStream;
#[cfg(not(feature = "mco"))]
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
#[cfg(not(feature = "mco"))]
pub type Sender<T> = crossbeam::channel::Sender<T>;
#[cfg(not(feature = "mco"))]
pub type JoinHandle<T> = std::thread::JoinHandle<T>;
#[cfg(not(feature = "mco"))]
pub type Mutex<T> = std::sync::Mutex<T>;
#[cfg(not(feature = "mco"))]
pub type WaitGroup = crossbeam_utils::sync::WaitGroup;

#[cfg(not(feature = "mco"))]
pub fn chan<T>() -> (Sender<T>, Receiver<T>) {
    crossbeam::channel::unbounded()
}

#[cfg(not(feature = "mco"))]
pub fn sleep(d: Duration) {
    std::thread::sleep(d)
}

#[cfg(not(feature = "mco"))]
pub fn spawn<F>(f: F) -> JoinHandle<()>
where
    F: FnOnce() + std::marker::Send + 'static,
{
    std::thread::spawn(f)
}

#[cfg(not(feature = "mco"))]
pub fn spawn_stack_size<F>(f: F, stack_size: usize) -> JoinHandle<()>
where
    F: FnOnce() + std::marker::Send + 'static,
{
    std::thread::spawn(f)
}
