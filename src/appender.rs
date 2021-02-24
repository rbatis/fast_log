use chrono::{DateTime, Local};
use log::Level;

/// LogAppender append logs
/// Appender will be running on single main thread,please do_log for new thread or new an Future
pub trait LogAppender: Send {
    fn do_log(&self, record: &FastLogRecord);

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    CommandRecord,
    CommandExit,
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
    pub now: DateTime<Local>,
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
pub trait RecordFormat: Send {
    fn do_format(&self, arg: &mut FastLogRecord);
}

pub struct FastLogFormatRecord {}

impl RecordFormat for FastLogFormatRecord {
    fn do_format(&self, arg: &mut FastLogRecord) {
        let data;
        let now = format!("{:36}", arg.now.to_string());
        match arg.level {
            Level::Warn | Level::Error => {
                data = format!(
                    "{} {} {} - {}  {}\n",
                    &now,
                    arg.level,
                    arg.module_path,
                    arg.args,
                    arg.format_line()
                );
            }
            _ => {
                data = format!(
                    "{} {} {} - {}\n",
                    &now, arg.level, arg.module_path, arg.args
                );
            }
        }
        arg.formated = data;
    }
}
