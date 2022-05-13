use chrono::{DateTime, Utc, Duration};
use log::{LevelFilter};
use std::time::SystemTime;
use std::ops::{Add, Sub};
use crossbeam_utils::sync::WaitGroup;
use crate::appender::Command::CommandRecord;

/// LogAppender append logs
/// Appender will be running on single main thread,please do_log for new thread or new an Future
pub trait LogAppender: Send {
    /// this method use one coroutines run this(Multiple appenders share one Appender).
    /// so. if you want  access the network, you can launch a coroutine using go! (| | {});
    fn do_log(&self, record: &FastLogRecord);

    /// flush or do nothing
    fn flush(&self) {}

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
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

impl RecordFormat for FastLogFormatRecord {
    fn do_format(&self, arg: &mut FastLogRecord) -> String {
        match arg.command {
            CommandRecord => {
                let now: DateTime<Utc> = chrono::DateTime::from(arg.now);
                let now = now.add(self.duration).naive_utc().to_string();
                let data = if arg.level.to_level_filter() <= self.display_file {
                    format!(
                        "{:26} {} {} - {}  {}:{}\n",
                        &now,
                        arg.level,
                        arg.module_path,
                        arg.args,
                        arg.file,
                        arg.line.unwrap_or_default()
                    )
                } else {
                    format!(
                        "{:26} {} {} - {}\n",
                        &now, arg.level, arg.module_path, arg.args
                    )
                };
                return data;
            }
            // fixme: can we use _ => { String::new() } instead ?
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
        String::new()
    }
}

impl FastLogFormatRecord {
    pub fn local_duration() -> Duration {
        let utc = chrono::Utc::now().naive_utc();
        let tz = chrono::Local::now().naive_local();
        tz.sub(utc)
    }

    pub fn new() -> FastLogFormatRecord {
        Self {
            duration: Self::local_duration(),
            display_file: LevelFilter::Warn,
        }
    }
}

impl Default for FastLogFormatRecord {
    fn default() -> Self {
        Self::new()
    }
}