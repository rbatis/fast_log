use std::sync::atomic::AtomicI32;
use crossbeam_channel::{Receiver, SendError, RecvError};
use log::{Level, Metadata, Record};
use parking_lot::RwLock;

use crate::appender::{Command, FastLogFormatRecord, FastLogRecord, LogAppender, RecordFormat};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::filter::{Filter, NoFilter};
use crate::plugin::console::ConsoleAppender;
use crate::plugin::file::FileAppender;
use crate::plugin::file_split::{FileSplitAppender, RollingType, Packer};
use crate::wait::FastLogWaitGroup;
use std::result::Result::Ok;
use std::time::{SystemTime, Duration};
use std::sync::Arc;

lazy_static! {
    static ref LOG_SENDER: RwLock<Option<LoggerSender>> = RwLock::new(Option::None);
}

pub struct LoggerSender {
    pub filter: Box<dyn Filter>,
    pub inner: crossbeam_channel::Sender<FastLogRecord>,
}

impl LoggerSender {
    pub fn new(cap: usize, filter: Box<dyn Filter>) -> (Self, Receiver<FastLogRecord>) {
        let (s, r) = crossbeam_channel::bounded(cap);
        (Self { inner: s, filter }, r)
    }
    pub fn send(&self, data: FastLogRecord) -> Result<(), SendError<FastLogRecord>> {
        self.inner.send(data)
    }
}

fn set_log(cup: usize, level: log::Level, filter: Box<dyn Filter>) -> Receiver<FastLogRecord> {
    LOGGER.set_level(level);
    let mut w = LOG_SENDER.write();
    let (log, recv) = LoggerSender::new(cup, filter);
    *w = Some(log);
    return recv;
}

pub struct Logger {
    level: AtomicI32,
}

impl Logger {
    pub fn set_level(&self, level: log::Level) {
        self.level
            .swap(level as i32, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_level(&self) -> log::Level {
        match self.level.load(std::sync::atomic::Ordering::Relaxed) {
            1 => Level::Error,
            2 => Level::Warn,
            3 => Level::Info,
            4 => Level::Debug,
            5 => Level::Trace,
            _ => panic!("error log level!"),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.get_level()
    }
    fn log(&self, record: &Record) {
        //send
        if let Some(sender) = LOG_SENDER.read().as_ref() {
            if !sender.filter.filter(record) {
                let fast_log_record = FastLogRecord {
                    command: Command::CommandRecord,
                    level: record.level(),
                    target: record.metadata().target().to_string(),
                    args: record.args().to_string(),
                    module_path: record.module_path().unwrap_or_default().to_string(),
                    file: record.file().unwrap_or_default().to_string(),
                    line: record.line().clone(),
                    now: SystemTime::now(),
                    formated: String::new(),
                };
                sender.send(fast_log_record);
            }
        }
    }
    fn flush(&self) {}
}

static LOGGER: Logger = Logger {
    level: AtomicI32::new(1),
};

/// initializes the log file path
/// log_file_path:  example->  "test.log"
/// channel_cup: example -> 1000
pub fn init_log(
    log_file_path: &str,
    channel_cup: usize,
    level: log::Level,
    mut filter: Option<Box<dyn Filter>>,
    debug_mode: bool,
) -> Result<FastLogWaitGroup, LogError> {
    let mut appenders: Vec<Box<dyn LogAppender>> = vec![Box::new(FileAppender::new(log_file_path))];
    if debug_mode {
        appenders.push(Box::new(ConsoleAppender {}));
    }
    let mut log_filter: Box<dyn Filter> = Box::new(NoFilter {});
    if filter.is_some() {
        log_filter = filter.take().unwrap();
    }
    return init_custom_log(
        appenders,
        channel_cup,
        level,
        log_filter,
        Box::new(FastLogFormatRecord::new()),
    );
}

/// initializes the log file path
/// log_dir_path:  example->  "log/"
/// channel_log_cup: example -> 1000
/// max_temp_size: do zip if temp log full
/// allow_zip_compress: zip compress log file
/// filter: log filter
/// packer: you can use ZipPacker or LZ4Packer or custom your Packer
pub fn init_split_log(
    log_dir_path: &str,
    channel_log_cup: usize,
    max_temp_size: LogSize,
    rolling_type: RollingType,
    level: log::Level,
    mut filter: Option<Box<dyn Filter>>,
    packer: Box<dyn Packer>,
    allow_console_log: bool,
) -> Result<FastLogWaitGroup, LogError> {
    let mut appenders: Vec<Box<dyn LogAppender>> = vec![Box::new(FileSplitAppender::new(
        log_dir_path,
        max_temp_size,
        rolling_type,
        1,
        packer,
    ))];
    if allow_console_log {
        appenders.push(Box::new(ConsoleAppender {}));
    }
    let mut log_filter: Box<dyn Filter> = Box::new(NoFilter {});
    if filter.is_some() {
        log_filter = filter.take().unwrap();
    }
    return init_custom_log(
        appenders,
        channel_log_cup,
        level,
        log_filter,
        Box::new(FastLogFormatRecord::new()),
    );
}

pub fn init_custom_log(
    appenders: Vec<Box<dyn LogAppender>>,
    log_cup: usize,
    level: log::Level,
    filter: Box<dyn Filter>,
    format: Box<dyn RecordFormat>,
) -> Result<FastLogWaitGroup, LogError> {
    if appenders.is_empty() {
        return Err(LogError::from("[fast_log] appenders can not be empty!"));
    }
    let wait_group = FastLogWaitGroup::new();
    let main_recv = set_log(log_cup, level, filter);

    let (back_sender, back_recv) = crossbeam_channel::bounded(log_cup * 10 * appenders.len());
    //main recv data
    let wait_group_main = wait_group.clone();
    std::thread::spawn(move || {
        loop {
            let data = main_recv.recv();
            if data.is_ok() {
                let mut s: FastLogRecord = data.unwrap();
                if s.command.eq(&Command::CommandExit) {
                    back_sender.send(s);
                    drop(wait_group_main);
                    break;
                }
                back_sender.send(s);
            }
        }
    });

    let wait_group_back = wait_group.clone();
    //back recv data
    std::thread::spawn(move || {
        loop {
            //recv
            let data = back_recv.recv();
            if let Ok(mut data) = data {
                if data.command.eq(&Command::CommandExit){
                    drop(wait_group_back);
                    break;
                }
                format.do_format(&mut data);
                for x in &appenders {
                    x.do_log(&mut data);
                }
            }
        }
    });

    let r = log::set_logger(&LOGGER).map(|()| log::set_max_level(level.to_level_filter()));
    if r.is_err() {
        return Err(LogError::from(r.err().unwrap()));
    } else {
        return Ok(wait_group);
    }
}

pub fn exit() -> Result<(), LogError> {
    let sender = LOG_SENDER.read();
    if sender.is_some() {
        let sender = sender.as_ref().unwrap();
        let fast_log_record = FastLogRecord {
            command: Command::CommandExit,
            level: log::Level::Info,
            target: String::new(),
            args: "exit".to_string(),
            module_path: String::new(),
            file: String::new(),
            line: None,
            now: SystemTime::now(),
            formated: "exit".to_string(),
        };
        let result = sender.send(fast_log_record);
        match result {
            Ok(()) => {
                return Ok(());
            }
            _ => {}
        }
    }

    return Err(LogError::E("[fast_log] exit fail!".to_string()));
}
