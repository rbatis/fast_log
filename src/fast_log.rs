use std::any::Any;
use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicI32;
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
use std::sync::mpsc::SendError;
use crossbeam_utils::sync::WaitGroup;
use once_cell::sync::{Lazy, OnceCell};
use crate::{chan, Receiver, Sender, spawn};
use crate::config::Config;

pub static LOG_SENDER: Lazy<LoggerSender> = Lazy::new(|| {
    LoggerSender::new_def()
});

pub struct LoggerSender {
    pub filter: OnceCell<Box<dyn Filter>>,
    pub send: Sender<FastLogRecord>,
    pub recv: Receiver<FastLogRecord>,
}

impl LoggerSender {
    pub fn new_def() -> Self {
        let (s, r) = chan();
        LoggerSender {
            filter: OnceCell::new(),
            send: s,
            recv: r,
        }
    }
    pub fn set_filter(&self, f: Box<dyn Filter>) {
        self.filter.get_or_init(|| f);
        self.filter.get();
    }
}

fn set_log(level: log::Level, filter: Box<dyn Filter>) {
    LOGGER.set_level(level);
    LOG_SENDER.set_filter(filter);
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
        let f = LOG_SENDER.filter.get();
        if f.is_some() {
            if !f.as_ref().unwrap().filter(record) {
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
                LOG_SENDER.send.send(fast_log_record);
            }
        }
    }
    fn flush(&self) {
        match flush(){
            Ok(v) => {
                v.wait();
            }
            Err(_) => {}
        }
    }
}

static LOGGER: Logger = Logger {
    level: AtomicI32::new(1),
};


pub fn init(config: Config) -> Result<&'static Logger, LogError> {
    if config.appenders.is_empty() {
        return Err(LogError::from("[fast_log] appenders can not be empty!"));
    }
    set_log(config.level.clone(), config.filter);
    //main recv data
    let appenders = config.appenders;
    let format = config.format;
    let level = config.level;
    std::thread::spawn(move || {
        let mut recever_vec = vec![];
        let mut sender_vec: Vec<Sender<Arc<FastLogRecord>>> = vec![];
        for a in appenders {
            let (s, r) = chan();
            sender_vec.push(s);
            recever_vec.push((r, a));
        }
        for (recever, appender) in recever_vec {
            spawn(move || {
                loop {
                    if let Ok(msg) = recever.recv() {
                        match msg.command{
                            Command::CommandRecord => {}
                            Command::CommandExit => {
                                break;
                            }
                            Command::CommandFlush(_) => {
                                appender.flush();
                                continue;
                            }
                        }
                        appender.do_log(msg.as_ref());
                    }
                }
            });
        }
        loop {
            //recv
            let data = LOG_SENDER.recv.recv();
            if let Ok(mut data) = data {
                data.formated = format.do_format(&mut data);
                let data = Arc::new(data);
                for x in &sender_vec {
                    x.send(data.clone());
                }
                match data.command{
                    Command::CommandRecord => {}
                    Command::CommandExit => {
                        break;
                    }
                    Command::CommandFlush(_) => {}
                }
            }
        }
    });
    let r = log::set_logger(&LOGGER).map(|()| log::set_max_level(level.to_level_filter()));
    if r.is_err() {
        return Err(LogError::from(r.err().unwrap()));
    } else {
        return Ok(&LOGGER);
    }
}

pub fn exit() -> Result<(), LogError> {
    let fast_log_record = FastLogRecord {
        command: Command::CommandExit,
        level: log::Level::Info,
        target: String::new(),
        args: String::new(),
        module_path: String::new(),
        file: String::new(),
        line: None,
        now: SystemTime::now(),
        formated: String::new(),
    };
    let result = LOG_SENDER.send.send(fast_log_record);
    match result {
        Ok(()) => {
            return Ok(());
        }
        _ => {}
    }
    return Err(LogError::E("[fast_log] exit fail!".to_string()));
}


pub fn flush() -> Result<WaitGroup, LogError> {
    let wg=WaitGroup::new();
    let fast_log_record = FastLogRecord {
        command: Command::CommandFlush(wg.clone()),
        level: log::Level::Info,
        target: String::new(),
        args: String::new(),
        module_path: String::new(),
        file: String::new(),
        line: None,
        now: SystemTime::now(),
        formated: String::new(),
    };
    let result = LOG_SENDER.send.send(fast_log_record);
    match result {
        Ok(()) => {
            return Ok(wg);
        }
        _ => {}
    }
    return Err(LogError::E("[fast_log] flush fail!".to_string()));
}
