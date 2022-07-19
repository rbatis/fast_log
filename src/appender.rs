use crate::appender::Command::CommandRecord;
use crate::date;
use crossbeam_utils::sync::WaitGroup;
use log::LevelFilter;
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime};

/// LogAppender append logs
/// Appender will be running on single main thread,please do_log for new thread or new an Future
pub trait LogAppender: Send {
    /// Batch write log, or do nothing
    fn do_logs(&self, records: &[FastLogRecord]);

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

impl Command {
    pub fn to_i32(&self) -> i32 {
        match self {
            Command::CommandRecord => 1,
            Command::CommandExit => 2,
            Command::CommandFlush(_) => 3,
        }
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.to_i32().eq(&other.to_i32())
    }
}

impl Eq for Command {}

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

pub struct FastLogFormat {
    // show line level
    pub display_line_level: log::LevelFilter,
}

impl RecordFormat for FastLogFormat {
    fn do_format(&self, arg: &mut FastLogRecord) -> String {
        match arg.command {
            CommandRecord => {
                let data;
                let now = fastdate::DateTime::now();
                if arg.level.to_level_filter() <= self.display_line_level {
                    data = format!(
                        "{:29} {} {} - {}  {}:{}\n",
                        &now,
                        arg.level,
                        arg.module_path,
                        arg.args,
                        arg.file,
                        arg.line.unwrap_or_default()
                    );
                } else {
                    data = format!(
                        "{:29} {} {} - {}\n",
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

impl FastLogFormat {
    pub fn new() -> FastLogFormat {
        Self {
            display_line_level: LevelFilter::Warn,
        }
    }

    ///show line level
    pub fn set_display_line_level(mut self, level: LevelFilter) -> Self {
        self.display_line_level = level;
        self
    }
}
