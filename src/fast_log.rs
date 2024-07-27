use crate::appender::{Command, FastLogRecord};
use crate::config::Config;
use crate::error::LogError;
use crate::{chan, spawn, Receiver, SendError, Sender, WaitGroup};
use log::{LevelFilter, Log, Metadata, Record};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

pub static LOGGER: OnceLock<Logger> = OnceLock::new();

/// get Logger,but you must call `fast_log::init`
pub fn logger() -> &'static Logger {
    LOGGER.get_or_init(|| { Logger::default() })
}

pub struct Logger {
    pub cfg: OnceLock<Config>,
    pub send: OnceLock<Sender<FastLogRecord>>,
    pub recv: OnceLock<Receiver<FastLogRecord>>,
}

impl Logger {
    pub fn default() -> Self {
        Self {
            cfg: OnceLock::default(),
            send: OnceLock::default(),
            recv: OnceLock::default(),
        }
    }

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
        if let Some(send) = LOGGER.get_or_init(|| { Logger::default() }).send.get() {
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
        if let Some(filter) = LOGGER.get_or_init(|| { Logger::default() }).cfg.get() {
            if let Some(send) = LOGGER.get_or_init(|| { Logger::default() }).send.get() {
                for filter in filter.filters.iter() {
                    if !filter.do_log(record) {
                        return;
                    }
                }
                let _ = send.send(FastLogRecord {
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
    LOGGER.get_or_init(|| { Logger::default() }).send.set(s).map_err(|_| LogError::from("set fail"))?;
    LOGGER.get_or_init(|| { Logger::default() }).recv.set(r).map_err(|_| LogError::from("set fail"))?;
    LOGGER.get_or_init(|| { Logger::default() }).set_level(config.level);
    LOGGER.get_or_init(|| { Logger::default() })
        .cfg
        .set(config)
        .map_err(|_| LogError::from("set fail="))?;
    //main recv data
    log::set_logger(LOGGER.get_or_init(|| { Logger::default() }))
        .map(|()| log::set_max_level(LOGGER.get_or_init(|| { Logger::default() }).cfg.get().unwrap().level))
        .map_err(|e| LogError::from(e))?;

    let mut receiver_vec = vec![];
    let mut sender_vec: Vec<Sender<Arc<Vec<FastLogRecord>>>> = vec![];
    let cfg = LOGGER.get_or_init(|| { Logger::default() }).cfg.get().unwrap();
    for a in cfg.appends.iter() {
        let (s, r) = chan(cfg.chan_len);
        sender_vec.push(s);
        receiver_vec.push((r, a));
    }
    for (receiver, appender) in receiver_vec {
        spawn(move || {
            let mut exit = false;
            loop {
                let mut remain = vec![];
                if receiver.len() == 0 {
                    if let Ok(msg) = receiver.recv() {
                        remain.push(msg);
                    }
                }
                //recv all
                loop {
                    match receiver.try_recv() {
                        Ok(v) => {
                            remain.push(v);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                //lock get appender
                let mut shared_appender = appender.lock();
                for msg in remain {
                    shared_appender.do_logs(msg.as_ref());
                    for x in msg.iter() {
                        match x.command {
                            Command::CommandRecord => {}
                            Command::CommandExit => {
                                exit = true;
                                continue;
                            }
                            Command::CommandFlush(_) => {
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
                let recv = LOGGER.get_or_init(|| { Logger::default() }).recv.get().unwrap();
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
                        LOGGER.get_or_init(|| { Logger::default() }).cfg.get().unwrap().format.do_format(x);
                    }
                    if x.command.eq(&Command::CommandExit) {
                        exit = true;
                    }
                }
                let data = Arc::new(remain);
                for x in senders.iter() {
                    let _ = x.send(data.clone());
                }
                if exit {
                    break;
                }
            }
        });
    }
    return Ok(LOGGER.get_or_init(|| { Logger::default() }));
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
    let result = LOGGER.get_or_init(|| { Logger::default() })
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
    let result = LOGGER.get_or_init(|| { Logger::default() })
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
    LOGGER.get_or_init(|| { Logger::default() }).print(log)
}
