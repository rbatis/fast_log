use crate::appender::{Command, FastLogRecord, RecordFormat};
use log::LevelFilter;
use std::time::Duration;

pub struct FastLogFormat {
    // show line level
    pub display_line_level: log::LevelFilter,
}

impl RecordFormat for FastLogFormat {
    fn do_format(&self, arg: &mut FastLogRecord) {
        match &arg.command {
            Command::CommandRecord => {
                let now = fastdate::DateTime::from(arg.now)
                    .add(Duration::from_secs(fastdate::offset_sec() as u64));
                if arg.level.to_level_filter() <= self.display_line_level {
                    arg.formated = format!(
                        "{} {} {}:{} {}\n",
                        &now,
                        arg.level,
                        arg.file,
                        arg.line.unwrap_or_default(),
                        arg.args,
                    );
                } else {
                    arg.formated = format!(
                        "{} {} {} - {}\n",
                        &now, arg.level, arg.module_path, arg.args
                    );
                }
            }
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
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

pub struct FastLogFormatJson {}

impl RecordFormat for FastLogFormatJson {
    fn do_format(&self, arg: &mut FastLogRecord) {
        match &arg.command {
            Command::CommandRecord => {
                let now = fastdate::DateTime::now();
                //{"args":"Commencing yak shaving","date":"2022-08-19 09:53:47.798674","file":"example/src/split_log.rs","level":"INFO","line":21}
                arg.formated = format!(
                    "{}\"args\":\"{}\",\"date\":\"{}\",\"file\":\"{}\",\"level\":\"{}\",\"line\":{}{}",
                    "{",
                    arg.args,
                    now,
                    arg.file,
                    arg.level,
                    arg.line.unwrap_or_default(),
                    "}\n"
                );
            }
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
    }
}

impl FastLogFormatJson {
    pub fn new() -> FastLogFormatJson {
        Self {}
    }
}
