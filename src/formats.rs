use crate::appender::{Command, FastLogRecord, RecordFormat};
use log::LevelFilter;

pub struct FastLogFormat {
    // show line level
    pub display_line_level: log::LevelFilter,
}

impl RecordFormat for FastLogFormat {
    fn do_format(&self, arg: &mut FastLogRecord) -> String {
        match &arg.command {
            Command::CommandRecord => {
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

pub struct FastLogFormatJson {}

impl RecordFormat for FastLogFormatJson {
    fn do_format(&self, arg: &mut FastLogRecord) -> String {
        match &arg.command {
            Command::CommandRecord => {
                let now = fastdate::DateTime::now();
                //{"args":"Commencing yak shaving","date":"2022-08-19 09:53:47.798674","file":"example/src/split_log.rs","level":"INFO","line":21}
                let js = format!(
                    "{}\"args\":\"{}\",\"date\":\"{}\",\"file\":\"{}\",\"level\":\"{}\",\"line\":{}{}",
                    "{",
                    arg.args,
                    now,
                    arg.file,
                    arg.level,
                    arg.line.unwrap_or_default(),
                    "}\n"
                );
                return js;
            }
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
        return String::new();
    }
}

impl FastLogFormatJson {
    pub fn new() -> FastLogFormatJson {
        Self {}
    }
}
