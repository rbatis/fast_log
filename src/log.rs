use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::RecvError;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use log::{error, info, warn};
use log::{Level, LevelFilter, Metadata, Record};
use tokio::prelude::*;

use crate::error::LogError;
use crate::time_util;

pub struct SimpleLogger {
    pub runtime_type: RuntimeType,

    //std sender
    pub std_sender: Option<std::sync::mpsc::SyncSender<String>>,
    //std recv
    pub std_recv: Option<Mutex<std::sync::mpsc::Receiver<String>>>,
}

///runtime Type
#[derive(Clone,Debug)]
pub enum RuntimeType {
    Std,
    TokIo,
    AsyncStd,
}


impl SimpleLogger {
    pub fn new(runtime_type: RuntimeType) -> Self {
        let (s, r) = std::sync::mpsc::sync_channel(1000);
        return Self {
            runtime_type,
            std_sender: Some(s),
            std_recv: Some(Mutex::new(r)),
        };
    }
    pub fn send(&self, arg: String) {
        match self.runtime_type {
            RuntimeType::Std => {
                self.std_sender.as_ref().unwrap().send(arg);
            }
            RuntimeType::TokIo => {
                self.std_sender.as_ref().unwrap().send(arg);
            }
            _ => { panic!(format!("[fast_log] un support send for type:{:?}",self.runtime_type)) }
        }
    }

    pub fn recv(&self) -> Result<String, RecvError> {
        match self.runtime_type {
            RuntimeType::Std => {
                self.std_recv.as_ref().unwrap().lock().unwrap().recv()
            }
            RuntimeType::TokIo => {
                self.std_recv.as_ref().unwrap().lock().unwrap().recv()
            }
            _ => { panic!(format!("[fast_log] un support recv for type:{:?}",self.runtime_type)) }
        }
    }
}

lazy_static! {
   static ref LOG:RwLock<Option<SimpleLogger>>= RwLock::new(None);
}

fn set_log(runtime_type: RuntimeType) {
    let mut n = LOG.write().unwrap();
    *n = Some(SimpleLogger::new(runtime_type));
}

fn log_recv() -> Option<String> {
    let lockResult = LOG.read();
    if lockResult.is_err() {
        println!("{}", lockResult.err().unwrap());
        return None;
    }
    let lockResult = lockResult.as_ref().unwrap();
    if lockResult.is_none() {
        println!("fast log not init");
        return None;
    }
    let data = lockResult.as_ref().unwrap().recv();
    if data.is_ok() {
        return Some(data.unwrap());
    }
    return None;
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

            //send
            let lockResult = LOG.read();
            if lockResult.is_err() {
                println!("{}", lockResult.err().unwrap());
                return;
            }
            let lockResult = lockResult.unwrap();
            if lockResult.is_none() {
                println!("fast log not init");
                return;
            }
            lockResult.as_ref().unwrap().send(data);
        }
    }
    fn flush(&self) {}
}

static LOGGER: Logger = Logger {};


/// initializes the log file path
/// log_file_path for example "test.log"
/// 初始化日志文件路径
/// log_file_path 文件路径 例如 "test.log"
pub fn init_log(log_file_path: &str, runtime_type: RuntimeType) -> Result<(), Box<dyn std::error::Error + Send>> {
    set_log(runtime_type);
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

            //recv
            let data = log_recv();
            if data.is_some() {
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
pub async fn init_async_log(log_file_path: &str, runtime_type: RuntimeType) -> Result<(), Box<dyn std::error::Error + Send>> {
    match runtime_type {
        RuntimeType::Std => {
            panic!("async log un support type Std! must use tokio and async_std");
        }
        _ => {}
    }
    set_log(runtime_type);
    let mut file = open_file(log_file_path).await;
    if file.is_err() {
        let e = LogError::from(format!("[log] open error! {}", file.err().unwrap().to_string().as_str()));
        return Err(Box::new(e));
    }
    let mut file = file.unwrap();
    tokio::spawn(async move {
        loop {
            //recv
            let data = log_recv();
            if data.is_some() {
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
    let file = tokio::fs::OpenOptions::new().write(true).create(true).open(log_file_path).await?;
    return Ok(file);
}


#[test]
pub fn test_log() {
    init_log("requests.log", RuntimeType::Std);
    // DEBUG_MODE.store(false,std::sync::atomic::Ordering::Relaxed);
    info!("Commencing yak shaving");
    std::thread::sleep(Duration::from_secs(5));
}


//cargo test --release --color=always --package fast_log --lib log::bench_log --all-features -- --nocapture --exact
#[test]
pub fn bench_log() {
    init_log("requests.log", RuntimeType::Std);
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
    init_async_log("requests.log", RuntimeType::TokIo).await;
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