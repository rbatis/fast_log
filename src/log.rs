use std::borrow::{Borrow, BorrowMut};
use std::error::Error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::intrinsics::write_bytes;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::RecvError;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local, Utc};
use log::{error, info, warn};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use tokio::prelude::*;

use crate::error::LogError;
use crate::time_util;

pub struct SimpleLogger {
    pub sender: std::sync::mpsc::SyncSender<String>,
    pub recv: Mutex<std::sync::mpsc::Receiver<String>>,
}



impl SimpleLogger {
    pub fn new() -> Self {
        let (s, r) = std::sync::mpsc::sync_channel(1000);
        return Self {
            sender: s,
            recv: Mutex::new(r)
        };
    }
    pub fn send(&self, arg: String) {
        self.sender.send(arg);
    }

    pub fn recv(&self) -> Result<String, RecvError> {
        self.recv.lock().unwrap().recv()
    }
}

lazy_static! {
   static ref LOG:SimpleLogger=SimpleLogger::new();
}

/// debug mode,true:print to console, false ,only write to file.
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(true);


pub struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut module = "";
            if record.module_path().is_some() {
                module = record.module_path().unwrap();
            }
            let local: DateTime<Local> = Local::now();
            let data = format!("{:?} {} {} - {}", local, record.level(), module, record.args());
            LOG.send(data);
        }
    }
    fn flush(&self) {}
}

static LOGGER: Logger = Logger {};


/// initializes the log file path
/// log_file_path for example "test.log"
/// 初始化日志文件路径
/// log_file_path 文件路径 例如 "test.log"
pub fn init_log(log_file_path: &str) -> Result<(), Box<dyn std::error::Error + Send>> {
    let log_path = log_file_path.to_owned();
    let mut file = OpenOptions::new().create(true).append(true).open(log_path.as_str());
    if file.is_err() {
        file = File::create(Path::new(log_path.as_str()));
    }
    if file.is_err() {
        println!("[log] the log path:{} is not true!", log_path.as_str());
        let e = LogError::from(format!("[log] the log path:{} is not true!", log_path.as_str()).as_str());
        return Err(Box::new(e));
    }
    let mut file = file.unwrap();
    std::thread::spawn(move || {
        loop {
            let data = LOG.recv();
            if data.is_ok() {
                let s: String = data.unwrap() + "\n";
                let debug = DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed);
                if debug {
                    print!("{}", s.as_str());
                }
                file.write(s.as_bytes());
                file.flush();
            }
        }
    });
    let r = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info));
    if r.is_err() {
        return Err(Box::new(r.err().unwrap()));
    } else {
        return Ok(());
    }
}


/// async write based on tokio
pub async fn init_async_log(log_file_path: &str) -> Result<(), Box<dyn std::error::Error + Send>> {
    let mut file = open_file(log_file_path).await;
    if file.is_err() {
        let e = LogError::from(format!("[log] open error! {}", file.err().unwrap().to_string().as_str()));
        return Err(Box::new(e));
    }
    let mut file = file.unwrap();
    tokio::spawn(async move {
        loop {
            let data = LOG.recv();
            if data.is_ok() {
                let s: String = data.unwrap() + "\n";
                let debug = DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed);
                if debug {
                    print!("{}", s.as_str());
                }
                file.write_all(s.as_bytes()).await;
                file.sync_data().await;
            }
        }
    });
    let r = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info));
    if r.is_err() {
        return Err(Box::new(r.err().unwrap()));
    } else {
        return Ok(());
    }
}


async fn open_file(log_file_path: &str) -> std::result::Result<tokio::fs::File, std::io::Error> {
    let mut file = tokio::fs::OpenOptions::new().write(true).create(true).open(log_file_path).await?;
    return Ok(file);
}


#[test]
pub fn test_log() {
    init_log("requests.log");
    // DEBUG_MODE.store(false,std::sync::atomic::Ordering::Relaxed);
    info!("Commencing yak shaving");
    std::thread::sleep(Duration::from_secs(5));
}


//cargo test --release --color=always --package fast_log --lib log::bench_log --all-features -- --nocapture --exact
#[test]
pub fn bench_log() {
    init_log("requests.log");
    // DEBUG_MODE.store(false,std::sync::atomic::Ordering::Relaxed);
    let total = 10000000;
    let now = SystemTime::now();
    for i in 0..total {
        //sleep(Duration::from_secs(1));
        info!("Commencing yak shaving");
    }
    time_util::count_time_tps(total, now);
}


//cargo test --release --color=always --package fast_log --lib log::bench_async_log --all-features -- --nocapture --exact
#[tokio::main]
#[test]
async fn bench_async_log() {
    init_async_log("requests.log").await;
    // DEBUG_MODE.store(false,std::sync::atomic::Ordering::Relaxed);
    let total = 10000000;
    let now = SystemTime::now();
    for i in 0..total {
        //sleep(Duration::from_secs(1));
        info!("Commencing yak shaving");
    }
    time_util::count_time_tps(total, now);
    sleep(Duration::from_secs(3600));
}