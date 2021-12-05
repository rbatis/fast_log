use chrono::{DateTime, Local, Utc, Timelike, Duration};
use log::Level;
use std::time::SystemTime;
use std::ops::{Add, Sub};

/// LogAppender append logs
/// Appender will be running on single main thread,please do_log for new thread or new an Future
pub trait LogAppender: Send {
    /// this method use one coroutines run this(Multiple appenders share one Appender).
    /// so. if you want  access the network, you can launch a coroutine using go! (| | {});
    fn do_log(&self, record: &mut FastLogRecord);

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    CommandRecord,
    CommandExit,
    /// Ensure that the log splitter forces splitting and saves the log
    CommandFlush
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

impl FastLogRecord {
    pub fn format_line(&self) -> String {
        match (self.file.as_str(), self.line.unwrap_or(0)) {
            (file, line) => format!("({}:{})", file, line),
        }
    }
}

/// format record data
pub trait RecordFormat: Send + Sync {
    fn do_format(&self, arg: &mut FastLogRecord);
}

pub struct FastLogFormatRecord {
    pub duration: Duration,
}

impl RecordFormat for FastLogFormatRecord {
    fn do_format(&self, arg: &mut FastLogRecord) {
        let data;
        let now: DateTime<Utc> = chrono::DateTime::from(arg.now);
        let now = now.add(self.duration);
        match arg.level {
            Level::Warn | Level::Error => {
                data = format!(
                    "{:36} {} {} - {}  {}\n",
                    &now,
                    arg.level,
                    arg.module_path,
                    arg.args,
                    arg.format_line()
                );
            }
            _ => {
                data = format!(
                    "{:36} {} {} - {}\n",
                    &now, arg.level, arg.module_path, arg.args
                );
            }
        }
        arg.formated = data;
    }
}

impl FastLogFormatRecord {
    pub fn new() -> FastLogFormatRecord {
        let utc = chrono::Utc::now().naive_utc();
        let tz = chrono::Local::now().naive_local();
        let d = tz.sub(utc);
        Self {
            duration: d
        }
    }
}
