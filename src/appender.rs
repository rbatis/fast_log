
use log::{LevelFilter};
use std::time::{Duration, SystemTime};
use std::ops::{Add, Sub};
use std::sync::Arc;
use crossbeam_utils::sync::WaitGroup;
use crate::appender::Command::CommandRecord;
use crate::date;

/// LogAppender append logs
/// Appender will be running on single main thread,please do_log for new thread or new an Future
pub trait LogAppender: Send {
    /// Batch write log, or do nothing
    fn do_logs(&self, records: &[FastLogRecord]) {
        for x in records {
            self.do_log(x);
        }
    }

    /// write one log, you can use record.formated write to file or any storage
    fn do_log(&self, record: &FastLogRecord);

    /// flush or do nothing
    fn flush(&self) {}
}

#[derive(Clone, Debug)]
pub enum Command {
    CommandRecord,
    CommandExit,
    /// Ensure that the log splitter forces splitting and saves the log
    CommandFlush(WaitGroup),
}

#[derive(Clone, Debug)]
pub struct FastLogRecord {
    pub command: Command,
    pub level: log::Level,
    pub target: String,
    pub args: String,
    pub module_path: String,
    pub file: String,
    pub line: Option<u32>,
    pub now: SystemTime,
    pub formated: String,
}

/// format record data
pub trait RecordFormat: Send + Sync {
    fn do_format(&self, arg: &mut FastLogRecord) -> String;
}

pub struct FastLogFormatRecord {
    pub duration: Duration,
    pub display_file: log::LevelFilter,
}

fn to_zero_left(arg:u8)->String{
    if arg<=9{
        return format!("0{}",arg);
    }else{
        return arg.to_string();
    }
}

impl RecordFormat for FastLogFormatRecord {
    fn do_format(&self, arg: &mut FastLogRecord) -> String {
        match arg.command {
            CommandRecord => {
                let data;
                let now = date::LogDate::from(arg.now.add(self.duration));
                if arg.level.to_level_filter() <= self.display_file {
                    data = format!(
                        "{:26} {} {} - {}  {}:{}\n",
                        &now,
                        arg.level,
                        arg.module_path,
                        arg.args,
                        arg.file,
                        arg.line.unwrap_or_default()
                    );
                } else {
                    data = format!(
                        "{:26} {} {} - {}\n",
                        &now, arg.level, arg.module_path, arg.args
                    );
                }
                return data;
            }
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
        return String::new();
    }
}

impl FastLogFormatRecord {

    pub fn local_duration() -> Duration {
        let utc = chrono::Utc::now().naive_utc();
        let tz = chrono::Local::now().naive_local();
        tz.sub(utc).to_std().unwrap_or_default()
    }

    pub fn new() -> FastLogFormatRecord {
        Self {
            duration: Self::local_duration(),
            display_file: LevelFilter::Warn,
        }
    }
}