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
    CommandFlush,
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
    fn do_format(&self, arg: &mut FastLogRecord)->String;
}

pub struct FastLogFormatRecord {
    pub duration: Duration,
}

impl RecordFormat for FastLogFormatRecord {
    fn do_format(&self, arg: &mut FastLogRecord) ->String{
        let data;
        //let now: DateTime<Utc> = chrono::DateTime::from(arg.now);
        let now = "asdfasdfasdfasdfsfda";
        match arg.level {
            Level::Warn | Level::Error => {
                data = format!(
                    "{:30} {} {} - {}  {}:{}\n",
                    &now,
                    arg.level,
                    arg.module_path,
                    arg.args,
                    arg.file,
                    arg.line.unwrap_or_default()
                );
            }
            _ => {
                data = format!(
                    "{:30} {} {} - {}\n",
                    &now, arg.level, arg.module_path, arg.args
                );
            }
        }
        data
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
            duration: Self::local_duration()
        }
    }
}

// #[cfg(test)]
// mod test {
//     use std::time::{Instant, SystemTime};
//     use log::Level;
//     use crate::appender::{Command, FastLogFormatRecord, FastLogRecord, RecordFormat};
//     use crate::bencher::QPS;
//
//     #[test]
//     fn test_bench() {
//         let arg = FastLogFormatRecord::new();
//         let mut a = FastLogRecord {
//             command: Command::CommandRecord,
//             level: Level::Error,
//             target: "".to_string(),
//             args: "".to_string(),
//             module_path: "".to_string(),
//             file: "".to_string(),
//             line: None,
//             now: SystemTime::now(),
//             formated: "".to_string(),
//         };
//         let total = 10000;
//         let now = Instant::now();
//         for index in 0..total {
//             //use Time: 5.6558ms ,each:565 ns/op
//             //use QPS: 1761897 QPS/s
//             arg.do_format(&mut a);
//         }
//         now.time(total);
//         now.qps(total);
//     }
// }