use crate::WaitGroup;
use std::time::SystemTime;

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
    fn do_format(&self, arg: &mut FastLogRecord);
}
