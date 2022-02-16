
use std::time::Duration;

/// if use mco runtime
#[cfg(feature = "mco")]
pub type TcpListener = mco::net::TcpListener;
#[cfg(feature = "mco")]
pub type TcpStream = mco::net::TcpStream;
#[cfg(feature = "mco")]
pub type Receiver<T> = mco::std::sync::channel::Receiver<T>;
#[cfg(feature = "mco")]
pub type Sender<T> = mco::std::sync::channel::Sender<T>;
#[cfg(feature = "mco")]
pub type JoinHandle<T> = mco::coroutine::JoinHandle<T>;
#[cfg(feature = "mco")]
pub type Mutex<T> = mco::std::sync::Mutex<T>;
#[cfg(feature = "mco")]
pub type WaitGroup = mco::std::sync::WaitGroup;

#[cfg(feature = "mco")]
pub fn chan<T>() -> (Sender<T>, Receiver<T>) {
    mco::chan!()
}

#[cfg(feature = "mco")]
pub fn sleep(d: Duration) {
    mco::coroutine::sleep(d)
}

#[cfg(feature = "mco")]
pub fn spawn<F>(f: F) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    mco::coroutine::Builder::new().stack_size(0x1000).spawn(f)
}

#[cfg(feature = "mco")]
pub fn spawn_stack_size<F>(f: F, stack_size:usize) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    mco::coroutine::Builder::new().stack_size(stack_size).spawn(f)
}


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
pub fn spawn<F>(f: F) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    std::thread::spawn(f)
}

#[cfg(not(feature = "mco"))]
pub fn spawn_stack_size<F>(f: F, stack_size:usize) -> JoinHandle<()> where F: FnOnce() + std::marker::Send + 'static {
    std::thread::spawn(f)
}