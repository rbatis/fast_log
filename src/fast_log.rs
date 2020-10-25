use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use crossbeam_channel::{bounded, RecvError, SendError};
use log::{Level, LevelFilter, Metadata, Record};
use log::info;

use crate::error::LogError;
use crate::time_util;

lazy_static! {
   static ref LOG_SENDER:RwLock<Option<LoggerSender>>=RwLock::new(Option::None);
}


pub struct LoggerRecv {
    //std recv
    pub recv: Option<crossbeam_channel::Receiver<String>>,
}

impl LoggerRecv {
    pub fn recv(&self) -> Result<String, RecvError> {
        self.recv.as_ref().unwrap().recv()
    }
}


pub struct LoggerSender {
    pub runtime_type: RuntimeType,
    //std sender
    pub std_sender: Option<crossbeam_channel::Sender<String>>,
}

impl LoggerSender {
    pub fn new(runtime_type: RuntimeType, cap: usize) -> (Self, LoggerRecv) {
        return match runtime_type {
            _ => {
                let (s, r) = bounded(cap);
                (Self {
                    runtime_type,
                    std_sender: Some(s),
                }, LoggerRecv { recv: Some(r) })
            }
        };
    }
    pub fn send(&self, data: &str) -> Result<(), SendError<String>> {
        self.std_sender.as_ref().unwrap().send(data.to_string())
    }
}

///runtime Type
#[derive(Clone, Debug)]
pub enum RuntimeType {
    Std
}


fn set_log(runtime_type: RuntimeType, cup: usize) -> LoggerRecv {
    let mut w = LOG_SENDER.write().unwrap();
    let (log, recv) = LoggerSender::new(runtime_type, cup);
    *w = Some(log);
    return recv;
}


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

            let data;

            let level = record.level();
            match level {
                Level::Warn | Level::Error => {
                    data = format!("{:?} {} {} - {}  {}\n", local, record.level(), module, record.args(), format_line(record));
                }
                _ => {
                    data = format!("{:?} {} {} - {}\n", local, record.level(), module, record.args());
                }
            }
            if !cfg!(feature = "no_print") {
                if cfg!(feature = "befor_print") {
                    print!("{}", &data);
                }
            }
            //send
            match LOG_SENDER.read() {
                Ok(lock) => {
                    match lock.is_some() {
                        true => {
                            lock.as_ref().unwrap().send(&data);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    fn flush(&self) {}
}

fn format_line(record: &Record<'_>) -> String {
    match (record.file(), record.line()) {
        (Some(file), Some(line)) => format!("({}:{})", file, line),
        _ => String::new(),
    }
}

static LOGGER: Logger = Logger {};


pub trait FastLog: Send {
    fn level(&self)->log::Level;
    fn do_log(&self, info: &str);
}


/// initializes the log file path
/// log_file_path for example "test.log"
/// 初始化日志文件路径
/// log_file_path 文件路径 例如 "test.log"
pub fn init_log(log_file_path: &str, cup: usize, custom_log: Option<Box<dyn FastLog>>) -> Result<(), Box<dyn std::error::Error + Send>> {
    let recv = set_log(RuntimeType::Std, cup);
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
            let data = recv.recv();
            if data.is_ok() {
                let s: String = data.unwrap();
                if !cfg!(feature = "no_print") {
                    if cfg!(feature = "after_print") {
                        print!("{}", &s);
                    }
                }
                if custom_log.is_none() {
                    file.write(s.as_bytes());
                    file.flush();
                } else {
                    custom_log.as_ref().unwrap().do_log(&s);
                }
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


//cargo test --release --color=always --package fast_log --lib log::bench_log --all-features -- --nocapture --exact
#[test]
pub fn bench_log() {
    init_log("requests.log", 1000, None);
    let total = 10000;
    let now = SystemTime::now();
    for index in 0..total {
        //sleep(Duration::from_secs(1));
        info!("Commencing yak shaving{}", index);
    }
    time_util::count_time_tps(total, now);
}