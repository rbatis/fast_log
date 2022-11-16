use crate::appender::{LogAppender, RecordFormat};
use crate::consts::LogSize;
use crate::filter::{Filter, NoFilter};
use crate::plugin::console::ConsoleAppender;
use crate::plugin::file::FileAppender;
use crate::plugin::file_loop::FileLoopAppender;
use crate::plugin::file_split::{FileSplitAppender, Packer, RollingType};
use crate::FastLogFormat;
use dark_std::sync::SyncVec;
use log::LevelFilter;
use std::sync::Mutex;

pub struct Config {
    pub appends: SyncVec<Mutex<Box<dyn LogAppender>>>,
    pub level: LevelFilter,
    pub filter: Box<dyn Filter>,
    pub format: Box<dyn RecordFormat>,
    pub chan_len: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appends: SyncVec::new(),
            level: LevelFilter::Trace,
            filter: Box::new(NoFilter {}),
            format: Box::new(FastLogFormat::new()),
            chan_len: Some(100000),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// set log LevelFilter
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }
    /// set log Filter
    pub fn filter<F: Filter + 'static>(mut self, filter: F) -> Self {
        self.filter = Box::new(filter);
        self
    }
    /// set log format
    pub fn format<F: RecordFormat + 'static>(mut self, format: F) -> Self {
        self.format = Box::new(format);
        self
    }
    /// add a ConsoleAppender
    pub fn console(self) -> Self {
        self.appends.push(Mutex::new(Box::new(ConsoleAppender {})));
        self
    }
    /// add a FileAppender
    pub fn file(self, file: &str) -> Self {
        self.appends
            .push(Mutex::new(Box::new(FileAppender::new(file))));
        self
    }
    /// add a FileLoopAppender
    pub fn file_loop(self, file: &str, max_temp_size: LogSize) -> Self {
        self.appends.push(Mutex::new(Box::new(FileLoopAppender::new(
            file,
            max_temp_size,
        ))));
        self
    }
    /// add a FileSplitAppender
    pub fn file_split<P: Packer + 'static>(
        self,
        file_path: &str,
        temp_size: LogSize,
        rolling_type: RollingType,
        packer: P,
    ) -> Self {
        self.appends
            .push(Mutex::new(Box::new(FileSplitAppender::new(
                file_path,
                temp_size,
                rolling_type,
                Box::new(packer),
            ))));
        self
    }
    /// add a custom LogAppender
    pub fn custom<Appender: LogAppender + 'static>(self, arg: Appender) -> Self {
        self.appends.push(Mutex::new(Box::new(arg)));
        self
    }

    /// if none=> unbounded() channel,if Some =>  bounded(len) channel
    pub fn chan_len(mut self, len: Option<usize>) -> Self {
        self.chan_len = len;
        self
    }
}
