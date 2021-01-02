use std::ops::Deref;
use std::sync::atomic::AtomicI32;
use std::sync::RwLock;

use chrono::Local;
use crossbeam_channel::{Receiver, SendError};
use log::{Level, Metadata, Record};

use crate::appender::{FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::filter::{Filter, NoFilter};
use crate::plugin::console::ConsoleAppender;
use crate::plugin::file::FileAppender;
use crate::plugin::file_split::{FileSplitAppender, RollingKeepType};

lazy_static! {
   static ref LOG_SENDER:RwLock<Option<LoggerSender>>=RwLock::new(Option::None);
}


pub struct LoggerSender {
    pub filter: Box<dyn Filter>,
    pub inner: crossbeam_channel::Sender<FastLogRecord>,
}

impl LoggerSender {
    pub fn new(runtime_type: RuntimeType, cap: usize, filter: Box<dyn Filter>) -> (Self, Receiver<FastLogRecord>) {
        return match runtime_type {
            _ => {
                let (s, r) = crossbeam_channel::bounded(cap);
                (Self {
                    inner: s,
                    filter,
                }, r)
            }
        };
    }
    pub fn send(&self, data: FastLogRecord) -> Result<(), SendError<FastLogRecord>> {
        self.inner.send(data)
    }
}

///runtime Type
#[derive(Clone, Debug)]
pub enum RuntimeType {
    Std
}


fn set_log(runtime_type: RuntimeType, cup: usize, level: log::Level, filter: Box<dyn Filter>) -> Receiver<FastLogRecord> {
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
        self.level.swap(level as i32, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_level(&self) -> log::Level {
        match self.level.load(std::sync::atomic::Ordering::Relaxed) {
            1 => Level::Error,
            2 => Level::Warn,
            3 => Level::Info,
            4 => Level::Debug,
            5 => Level::Trace,
            _ => panic!("error log level!")
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
                Ok(lock) => {
                    match lock.deref() {
                        Some(sender) => {
                            if !sender.filter.filter(record) {
                                let mut fast_log_record = FastLogRecord {
                                    level,
                                    target: record.metadata().target().to_string(),
                                    args: record.args().to_string(),
                                    module_path: record.module_path().unwrap_or("").to_string(),
                                    file: record.file().unwrap_or("").to_string(),
                                    line: record.line().clone(),
                                    now: Local::now(),
                                    formated: "".to_string(),
                                };
                                fast_log_record.set_formated();
                                sender.send(fast_log_record);
                            }
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

static LOGGER: Logger = Logger { level: AtomicI32::new(1) };


/// initializes the log file path
/// log_file_path:  example->  "test.log"
/// channel_cup: example -> 1000
pub fn init_log(log_file_path: &str, channel_cup: usize, level: log::Level, mut filter: Option<Box<dyn Filter>>, debug_mode: bool) -> Result<(), Box<dyn std::error::Error + Send>> {
    let mut appenders: Vec<Box<dyn LogAppender>> = vec![
        Box::new(FileAppender::new(log_file_path))
    ];
    if debug_mode {
        appenders.push(Box::new(ConsoleAppender {}));
    }
    let mut log_filter: Box<dyn Filter> = Box::new(NoFilter {});
    if filter.is_some() {
        log_filter = filter.take().unwrap();
    }
    return init_custom_log(appenders, channel_cup, level, log_filter);
}

/// initializes the log file path
/// log_dir_path:  example->  "log/"
/// channel_log_cup: example -> 1000
/// max_temp_size: do zip if temp log full
/// allow_zip_compress: zip compress log file
/// filter: log filter
pub fn init_split_log(log_dir_path: &str, channel_log_cup: usize, max_temp_size: LogSize, allow_zip_compress: bool,rolling_type:RollingKeepType, level: log::Level, mut filter: Option<Box<dyn Filter>>, debug_mode: bool) -> Result<(), Box<dyn std::error::Error + Send>> {
    let mut appenders: Vec<Box<dyn LogAppender>> = vec![
        Box::new(FileSplitAppender::new(log_dir_path, max_temp_size, rolling_type,allow_zip_compress,1))
    ];
    if debug_mode {
        appenders.push(Box::new(ConsoleAppender {}));
    }
    let mut log_filter: Box<dyn Filter> = Box::new(NoFilter {});
    if filter.is_some() {
        log_filter = filter.take().unwrap();
    }
    return init_custom_log(appenders, channel_log_cup, level, log_filter);
}

pub fn init_custom_log(appenders: Vec<Box<dyn LogAppender>>, log_cup: usize, level: log::Level, filter: Box<dyn Filter>) -> Result<(), Box<dyn std::error::Error + Send>> {
    if appenders.is_empty() {
        return Err(Box::new(LogError::from("[fast_log] appenders can not be empty!")));
    }
    let main_recv = set_log(RuntimeType::Std, log_cup, level, filter);
    if appenders.len() == 1 {
        //main recv data
        std::thread::spawn(move || {
            loop {
                let data = main_recv.recv();
                if data.is_ok() {
                    let s: FastLogRecord = data.unwrap();
                    for x in &appenders {
                        x.do_log(&s);
                    }
                }
            }
        });
    } else {
        let mut recvs = vec![];
        let mut sends = vec![];
        for idx in 0..appenders.len() {
            let (s, r) = crossbeam_channel::bounded(log_cup);
            recvs.push(r);
            sends.push(s);
        }
        //main recv data
        std::thread::spawn(move || {
            loop {
                let data = main_recv.recv();
                if data.is_ok() {
                    let s: FastLogRecord = data.unwrap();
                    for x in &sends {
                        x.send(s.clone());
                    }
                }
            }
        });

        //all appender recv
        let mut index = 0;
        for item in appenders {
            let recv = recvs[index].to_owned();
            std::thread::spawn(move || {
                loop {
                    //recv
                    let data = recv.recv();
                    if data.is_ok() {
                        let s: FastLogRecord = data.unwrap();
                        item.do_log(&s);
                    }
                }
            });
            index += 1;
        }
    }

    let r = log::set_logger(&LOGGER).map(|()| log::set_max_level(level.to_level_filter()));
    if r.is_err() {
        return Err(Box::new(r.err().unwrap()));
    } else {
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use log::{debug, info, Level};
    use log::error;

    use crate::{init_custom_log, init_log, init_split_log};
    use crate::bencher::QPS;
    use crate::consts::LogSize;
    use crate::fast_log::{FastLogRecord, LogAppender};
    use crate::filter::NoFilter;
    use crate::plugin::file_split::RollingKeepType;

    #[test]
    pub fn test_log() {
        init_log("requests.log", 1000, log::Level::Debug, None, true);
        debug!("Commencing yak shaving{}", 0);
        sleep(Duration::from_secs(1));
    }

    //cargo test --release --color=always --package fast_log --lib fast_log::test::bench_log --no-fail-fast -- --exact -Z unstable-options --show-output
    #[test]
    pub fn bench_log() {
        init_log("requests.log", 1000, log::Level::Info, None, false);
        let total = 10000;
        let now = Instant::now();
        for index in 0..total {
            //sleep(Duration::from_secs(1));
            info!("Commencing yak shaving{}", index);
        }
        now.time(total);
        now.qps(total);
        sleep(Duration::from_secs(1));
    }

    struct CustomLog {}

    impl LogAppender for CustomLog {
        fn do_log(&self, record: &FastLogRecord) {
            let data;
            match record.level {
                Level::Warn | Level::Error => {
                    data = format!("{} {} {} - {}  {}\n", &record.now, record.level, record.module_path, record.args, record.format_line());
                }
                _ => {
                    data = format!("{} {} {} - {}\n", &record.now, record.level, record.module_path, record.args);
                }
            }
            print!("{}", data);
        }
    }

    #[test]
    pub fn test_custom() {
        init_custom_log(vec![Box::new(CustomLog {})], 1000, log::Level::Info, Box::new(NoFilter {}));
        info!("Commencing yak shaving");
        error!("Commencing error");
        sleep(Duration::from_secs(1));
    }

    #[test]
    pub fn test_file_compation() {
        init_split_log("target/logs/", 1000, LogSize::MB(1), false, RollingKeepType::All,log::Level::Info, None, true);
        for _ in 0..20000 {
            info!("Commencing yak shaving");
        }
        sleep(Duration::from_secs(1));
    }

    #[test]
    pub fn test_file_compation_zip() {
        init_split_log("target/logs/", 1000, LogSize::KB(50), true, RollingKeepType::KeepNum(5),log::Level::Info, None, true);
        for _ in 0..20000 {
            info!("Commencing yak shaving");
        }
        sleep(Duration::from_secs(10));
    }

    #[test]
    pub fn test_file_compation_zip_stable_test() {
        init_split_log("target/logs/", 1000, LogSize::MB(100), true, RollingKeepType::All,log::Level::Info, None, false);
        let now = std::time::Instant::now();
        loop {
            info!("Commencing yak shaving");
            if now.elapsed() > Duration::from_secs(30) {
                break;
            }
        }
        info!("done");
        sleep(Duration::from_secs(100));
    }


    struct BenchRecvLog {}

    impl LogAppender for BenchRecvLog {
        fn do_log(&self, record: &FastLogRecord) {}
    }

    //cargo test --release --color=always --package fast_log --lib fast_log::test::bench_recv --no-fail-fast -- --exact -Z unstable-options --show-output
    #[test]
    pub fn bench_recv() {
        init_custom_log(vec![Box::new(BenchRecvLog {})], 1000, log::Level::Info, Box::new(NoFilter {}));
        let total = 10000;
        let now = Instant::now();
        for index in 0..total {
            //sleep(Duration::from_secs(1));
            info!("Commencing yak shaving{}", index);
        }
        now.time(total);
        now.qps(total);
        sleep(Duration::from_secs(1));
    }
}