use std::borrow::{Borrow, BorrowMut};
use std::fs;
use std::fs::{File, OpenOptions};
use std::intrinsics::write_bytes;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::sync::mpsc::RecvError;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc, Local};
use log::{error, info, warn};

use crate::time_util;
use std::error::Error;
use crate::error::LogError;

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

pub struct SimpleLogger {
    pub sender: std::sync::mpsc::Sender<String>,
    pub recv: std::sync::mpsc::Receiver<String>,
}

unsafe impl Send for SimpleLogger {}

unsafe impl Sync for SimpleLogger {}

impl SimpleLogger {
    pub fn new() -> Self {
        let (s, r) = std::sync::mpsc::channel();
        return Self {
            sender: s,
            recv: r,
        };
    }
    pub fn send(&self, arg: String) {
        self.sender.send(arg);
    }

    pub fn recv(&self) -> Result<String, RecvError> {
        self.recv.recv()
    }
}

lazy_static! {
   static ref LOG:SimpleLogger=SimpleLogger::new();
}

pub struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let local: DateTime<Local> = Local::now();
            let data = format!("{} - {} - {}", local, record.level(), record.args());
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
pub fn init_log(log_file_path: &str) -> Result<(), Box<dyn std::error::Error+Send>> {
    let log_path = log_file_path.to_owned();
    let mut file = OpenOptions::new().create(true).append(true).open(log_path.as_str());
    if file.is_err() {
        file = File::create(Path::new(log_path.as_str()));
    }
    if file.is_err() {
        println!("[log] the log path:{} is not true!", log_path.as_str());
        let e= LogError::from(format!("[log] the log path:{} is not true!", log_path.as_str()).as_str());
         return Err(Box::new(e));
    }
    let mut file = file.unwrap();
    std::thread::spawn(move || {
        loop {
            let data = LOG.recv();
            if data.is_ok() {
                let s: String = data.unwrap() + "\n";
                print!("{}", s.as_str());
                file.write(s.as_bytes());
                file.flush();
            }
        }
    });
    let r=log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info));
    if r.is_err(){
        return Err(Box::new(r.err().unwrap()));
    }else{
        return Ok(());
    }
}


#[test]
pub fn test_log() {
    println!("{:?}", SystemTime::now());

    init_log("requests.log");
    info!("Commencing yak shaving");
    std::thread::sleep(Duration::from_secs(5));
}


//cargo.exe test --release --color=always --package rbatis --lib log::bench_log --all-features -- --nocapture --exact
#[test]
pub fn bench_log() {
    init_log("requests.log");
    let total = 100000;
    let now = SystemTime::now();
    for i in 0..total {
        //sleep(Duration::from_secs(1));
        info!("Commencing yak shaving");
    }
    time_util::count_time_tps(total, now);
}