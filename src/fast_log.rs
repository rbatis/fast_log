use crate::appender::{Command, FastLogRecord};
use crate::config::Config;
use crate::error::LogError;
use crate::{chan, spawn, Receiver, SendError, Sender, WaitGroup};
use log::{LevelFilter, Log, Metadata, Record};
use once_cell::sync::{Lazy, OnceCell};
use std::ops::Deref;
use std::sync::Arc;
use std::time::SystemTime;

pub static LOGGER: Lazy<Logger> = Lazy::new(|| Logger {
    cfg: OnceCell::new(),
    send: OnceCell::new(),
    recv: OnceCell::new(),
});

pub struct Logger {
    pub cfg: OnceCell<Config>,
    pub send: OnceCell<Sender<FastLogRecord>>,
    pub recv: OnceCell<Receiver<FastLogRecord>>,
}

impl Logger {
    pub fn set_level(&self, level: LevelFilter) {
        log::set_max_level(level);
    }

    pub fn get_level(&self) -> LevelFilter {
        log::max_level()
    }

    /// print no other info
    pub fn print(&self, log: String) -> Result<(), SendError<FastLogRecord>> {
        let fast_log_record = FastLogRecord {
            command: Command::CommandRecord,
            level: log::Level::Info,
            target: "".to_string(),
            args: "".to_string(),
            module_path: "".to_string(),
            file: "".to_string(),
            line: None,
            now: SystemTime::now(),
            formated: log,
        };
        if let Some(send) = LOGGER.send.get() {
            send.send(fast_log_record)
        } else {
            // Ok(())
            Err(crossbeam_channel::SendError(fast_log_record))
        }
    }

    pub fn wait(&self) {
        self.flush();
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.get_level()
    }
    fn log(&self, record: &Record) {
        if let Some(filter) = LOGGER.cfg.get() {
            if let Some(send) = LOGGER.send.get() {
                if !filter.filter.filter(record) {
                    send.send(FastLogRecord {
                        command: Command::CommandRecord,
                        level: record.level(),
                        target: record.metadata().target().to_string(),
                        args: record.args().to_string(),
                        module_path: record.module_path().unwrap_or_default().to_string(),
                        file: record.file().unwrap_or_default().to_string(),
                        line: record.line().clone(),
                        now: SystemTime::now(),
                        formated: String::new(),
                    });
                }
            }
        }
    }
    fn flush(&self) {
        match flush() {
            Ok(v) => {
                v.wait();
            }
            Err(_) => {}
        }
    }
}

pub fn init(config: Config) -> Result<&'static Logger, LogError> {
    if config.appends.is_empty() {
        return Err(LogError::from("[fast_log] appends can not be empty!"));
    }
    let (s, r) = chan(config.chan_len);
    LOGGER.send.set(s).map_err(|e| LogError::from("set fail"))?;
    LOGGER.recv.set(r).map_err(|e| LogError::from("set fail"))?;
    LOGGER.set_level(config.level);
    LOGGER
        .cfg
        .set(config)
        .map_err(|e| LogError::from("set fail"))?;
    //main recv data
    log::set_logger(LOGGER.deref())
        .map(|()| log::set_max_level(LOGGER.cfg.get().unwrap().level))
        .map_err(|e| LogError::from(e))?;

    let mut recever_vec = vec![];
    let mut sender_vec: Vec<Sender<Arc<Vec<FastLogRecord>>>> = vec![];
    let cfg = LOGGER.cfg.get().unwrap();
    for a in cfg.appends.iter() {
        let (s, r) = chan(cfg.chan_len);
        sender_vec.push(s);
        recever_vec.push((r, a));
    }
    for (recever, appender) in recever_vec {
        spawn(move || {
            let mut exit = false;
            loop {
                let mut remain = vec![];
                if recever.len() == 0 {
                    if let Ok(msg) = recever.recv() {
                        remain.push(msg);
                    }
                }
                //recv all
                loop {
                    match recever.try_recv() {
                        Ok(v) => {
                            remain.push(v);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                let append = appender.lock();
                for msg in remain {
                    append.do_logs(msg.as_ref());
                    for x in msg.iter() {
                        match x.command {
                            Command::CommandRecord => {}
                            Command::CommandExit => {
                                exit = true;
                                continue;
                            }
                            Command::CommandFlush(_) => {
                                append.flush();
                                continue;
                            }
                        }
                    }
                }
                if exit {
                    break;
                }
            }
        });
    }
    let sender_vec = Arc::new(sender_vec);
    for _ in 0..1 {
        let senders = sender_vec.clone();
        spawn(move || {
            loop {
                let recv = LOGGER.recv.get().unwrap();
                let mut remain = Vec::with_capacity(recv.len());
                //recv
                if recv.len() == 0 {
                    if let Ok(item) = recv.recv() {
                        remain.push(item);
                    }
                }
                //recv all
                loop {
                    match recv.try_recv() {
                        Ok(v) => {
                            remain.push(v);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                let mut exit = false;
                for x in &mut remain {
                    if x.formated.is_empty() {
                        LOGGER.cfg.get().unwrap().format.do_format(x);
                    }
                    if x.command.eq(&Command::CommandExit) {
                        exit = true;
                    }
                }
                let data = Arc::new(remain);
                for x in senders.iter() {
                    x.send(data.clone());
                }
                if exit {
                    break;
                }
            }
        });
    }
    return Ok(LOGGER.deref());
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
    let result = LOGGER
        .send
        .get()
        .ok_or_else(|| LogError::from("not init"))?
        .send(fast_log_record);
    match result {
        Ok(()) => {
            return Ok(());
        }
        _ => {}
    }
    return Err(LogError::E("[fast_log] exit fail!".to_string()));
}

pub fn flush() -> Result<WaitGroup, LogError> {
    let wg = WaitGroup::new();
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
    let result = LOGGER
        .send
        .get()
        .ok_or_else(|| LogError::from("not init"))?
        .send(fast_log_record);
    match result {
        Ok(()) => {
            return Ok(wg);
        }
        _ => {}
    }
    return Err(LogError::E("[fast_log] flush fail!".to_string()));
}

pub fn print(log: String) -> Result<(), SendError<FastLogRecord>> {
    LOGGER.print(log)
}
