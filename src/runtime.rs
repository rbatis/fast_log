
use std::time::Duration;

/// if use cogo runtime
#[cfg(feature = "cogo")]
pub type TcpListener = cogo::net::TcpListener;
#[cfg(feature = "cogo")]
pub type TcpStream = cogo::net::TcpStream;
#[cfg(feature = "cogo")]
pub type Receiver<T> = cogo::std::sync::channel::Receiver<T>;
#[cfg(feature = "cogo")]
pub type Sender<T> = cogo::std::sync::channel::Sender<T>;
#[cfg(feature = "cogo")]
pub type JoinHandle<T> = cogo::coroutine::JoinHandle<T>;
#[cfg(feature = "cogo")]
pub type Mutex<T> = cogo::std::sync::Mutex<T>;
#[cfg(feature = "cogo")]
pub type WaitGroup = cogo::std::sync::WaitGroup;

#[cfg(feature = "cogo")]
pub fn chan<T>() -> (Sender<T>, Receiver<T>) {
    cogo::chan!()
}

#[cfg(feature = "cogo")]
pub fn sleep(d: Duration) {
    cogo::coroutine::sleep(d)
}

#[cfg(feature = "cogo")]
pub fn spawn<F>(f: F) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    cogo::coroutine::Builder::new().stack_size(0x1000).spawn(f)
}

#[cfg(feature = "cogo")]
pub fn spawn_stack_size<F>(f: F, stack_size:usize) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    cogo::coroutine::Builder::new().stack_size(stack_size).spawn(f)
}


/// if not cogo
#[cfg(not(feature = "cogo"))]
pub type TcpListener = std::net::TcpListener;
#[cfg(not(feature = "cogo"))]
pub type TcpStream = std::net::TcpStream;
#[cfg(not(feature = "cogo"))]
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
#[cfg(not(feature = "cogo"))]
pub type Sender<T> = crossbeam::channel::Sender<T>;
#[cfg(not(feature = "cogo"))]
pub type JoinHandle<T> = std::thread::JoinHandle<T>;
#[cfg(not(feature = "cogo"))]
pub type Mutex<T> = std::sync::Mutex<T>;
#[cfg(not(feature = "cogo"))]
pub type WaitGroup = crossbeam_utils::sync::WaitGroup;

#[cfg(not(feature = "cogo"))]
pub fn chan<T>() -> (Sender<T>, Receiver<T>) {
    crossbeam::channel::unbounded()
}

#[cfg(not(feature = "cogo"))]
pub fn sleep(d: Duration) {
    std::thread::sleep(d)
}

#[cfg(not(feature = "cogo"))]
pub fn spawn<F>(f: F) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    std::thread::spawn(f)
}

#[cfg(not(feature = "cogo"))]
pub fn spawn_stack_size<F>(f: F, stack_size:usize) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    std::thread::spawn(f)
}