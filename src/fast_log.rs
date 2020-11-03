use std::sync::atomic::AtomicI32;
use std::sync::RwLock;

use chrono::{DateTime, Local};
use crossbeam_channel::{select, Receiver, Select, SendError, Sender};
use log::{Level, LevelFilter, Metadata, Record};

use crate::filter::{Filter, ModuleFilter, NoFilter};
use crate::plugin::console::ConsoleAppender;
use crate::plugin::file::FileAppender;

lazy_static! {
    static ref LOG_SENDER: RwLock<Option<LoggerSender>> = RwLock::new(Option::None);
}

#[derive(Clone, Debug)]
pub struct FastLogRecord {
    pub level: log::Level,
    pub target: String,
    pub args: String,
    pub module_path: String,
    pub file: String,
    pub line: Option<u32>,
    pub now: DateTime<Local>,
}

impl FastLogRecord {
    pub fn format_line(&self) -> String {
        match (self.file.as_str(), self.line.unwrap_or(0)) {
            (file, line) => format!("({}:{})", file, line),
        }
    }
}

pub struct LoggerSender {
    pub runtime_type: RuntimeType,

    pub filter: Box<dyn Filter>,
    //std sender
    pub std_sender: Option<crossbeam_channel::Sender<FastLogRecord>>,
}

impl LoggerSender {
    pub fn new(
        runtime_type: RuntimeType,
        cap: usize,
        filter: Box<dyn Filter>,
    ) -> (Self, Receiver<FastLogRecord>) {
        return match runtime_type {
            _ => {
                let (s, r) = crossbeam_channel::bounded(cap);
                (
                    Self {
                        runtime_type,
                        std_sender: Some(s),
                        filter,
                    },
                    r,
                )
            }
        };
    }
    pub fn send(&self, data: FastLogRecord) -> Result<(), SendError<FastLogRecord>> {
        self.std_sender.as_ref().unwrap().send(data)
    }
}

///runtime Type
#[derive(Clone, Debug)]
pub enum RuntimeType {
    Std,
}

fn set_log(
    runtime_type: RuntimeType,
    cup: usize,
    level: log::Level,
    filter: Box<dyn Filter>,
) -> Receiver<FastLogRecord> {
    LOGGER.set_level(level);
    let mut w = LOG_SENDER.write().unwrap();
    let (log, recv) = LoggerSender::new(runtime_type, cup, filter);
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
        if self.enabled(record.metadata()) {
            let level = record.level();
            if self.get_level() < level {
                return;
            }
            //send
            match LOG_SENDER.read() {
                Ok(lock) => match lock.is_some() {
                    true => {
                        let sender = lock.as_ref().unwrap();
                        if !sender.filter.filter(record) {
                            sender.send(FastLogRecord {
                                level,
                                target: record.metadata().target().to_string(),
                                args: record.args().to_string(),
                                module_path: record.module_path().unwrap_or("").to_string(),
                                file: record.file().unwrap_or("").to_string(),
                                line: record.line().clone(),
                                now: Local::now(),
                            });
                        }
                    }
                    _ => {}
                },
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

static LOGGER: Logger = Logger {
    level: AtomicI32::new(1),
};

pub trait LogAppender: Send {
    fn do_log(&mut self, record: &FastLogRecord);
}

pub struct WaitFinish(Sender<()>);
impl Drop for WaitFinish {
    fn drop(&mut self) {
        self.0.send(());
    }
}

/// initializes the log file path
/// log_file_path:  example->  "test.log"
/// log_cup: example -> 1000
/// custom_log: default None
pub fn init_log(
    log_file_path: &str,
    log_cup: usize,
    level: log::Level,
    debug_mode: bool,
) -> Result<WaitFinish, Box<dyn std::error::Error + Send>> {
    let mut appenders: Vec<Box<dyn LogAppender>> = vec![Box::new(FileAppender::new(log_file_path))];
    if debug_mode {
        appenders.push(Box::new(ConsoleAppender {}));
    }
    return init_custom_log(appenders, log_cup, level, Box::new(NoFilter {}));
}

pub fn init_custom_log(
    mut appenders: Vec<Box<dyn LogAppender>>,
    log_cup: usize,
    level: log::Level,
    filter: Box<dyn Filter>,
) -> Result<WaitFinish, Box<dyn std::error::Error + Send>> {
    let recv = set_log(RuntimeType::Std, log_cup, level, filter);
    let (done, done_recv) = crossbeam_channel::bounded::<()>(0);

    let mut do_log = move |data| {
        let s: FastLogRecord = data;
        for x in &mut appenders {
            x.do_log(&s);
        }
    };

    std::thread::spawn(move || {
        let mut sel = Select::new();
        let log_index = sel.recv(&recv);
        let done_index = sel.recv(&done_recv);
        loop {
            let index = sel.ready();

            // 进程退出前， 将未写完的日志写完
            if index == done_index {
                while let Ok(data) = recv.try_recv() {
                    do_log(data);
                }
                break;
            }

            if let Ok(data) = recv.try_recv() {
                do_log(data);

            }
        }

        done_recv.try_recv();
    });
    let r = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
    if r.is_err() {
        return Err(Box::new(r.err().unwrap()));
    } else {
        return Ok(WaitFinish(done));
    }
}

#[cfg(test)]
mod test {
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    use log::error;
    use log::{info, Level};

    use crate::fast_log::{FastLogRecord, LogAppender};
    use crate::filter::{ModuleFilter, NoFilter};
    use crate::{init_custom_log, init_log, time_util};

    #[test]
    pub fn test_log() {
        init_log("requests.log", 1000, log::Level::Info, true);
        info!("Commencing yak shaving{}", 0);
        sleep(Duration::from_secs(1));
    }

    //cargo test --release --color=always --package fast_log --lib fast_log::test::bench_log --no-fail-fast -- --exact -Z unstable-options --show-output
    #[test]
    pub fn bench_log() {
        init_log("requests.log", 1000, log::Level::Info, false);
        let total = 10000;
        let now = SystemTime::now();
        for index in 0..total {
            //sleep(Duration::from_secs(1));
            info!("Commencing yak shaving{}", index);
        }
        time_util::count_time_tps(total, now);
        sleep(Duration::from_secs(1));
    }

    struct CustomLog {}

    impl LogAppender for CustomLog {
        fn do_log(&mut self, record: &FastLogRecord) {
            let mut data;
            match record.level {
                Level::Warn | Level::Error => {
                    data = format!(
                        "{} {} {} - {}  {}\n",
                        &record.now,
                        record.level,
                        record.module_path,
                        record.args,
                        record.format_line()
                    );
                }
                _ => {
                    data = format!(
                        "{} {} {} - {}\n",
                        &record.now, record.level, record.module_path, record.args
                    );
                }
            }
            print!("{}", data);
        }
    }

    #[test]
    pub fn test_custom() {
        init_custom_log(
            vec![Box::new(CustomLog {})],
            1000,
            log::Level::Info,
            Box::new(NoFilter {}),
        );
        info!("Commencing yak shaving");
        error!("Commencing error");
        sleep(Duration::from_secs(1));
    }
}
