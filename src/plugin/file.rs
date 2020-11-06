use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;

use log::Level;

use crate::fast_log::{FastLogRecord, LogAppender};
use futures_core::future::BoxFuture;

/// only write append into file
pub struct FileAppender {
    file: RefCell<File>
}

impl FileAppender {
    pub fn new(log_file_path: &str) -> FileAppender {
        Self {
            file: RefCell::new(OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file_path)
                .unwrap())
        }
    }
}

impl LogAppender for FileAppender {
    fn do_log(&self, record: &FastLogRecord) -> Option<BoxFuture<()>>{
        let mut data = String::new();
        match record.level {
            Level::Warn | Level::Error => {
                data = format!("{} {} {} - {}  {}\n", &record.now, record.level, record.module_path, record.args, record.format_line());
            }
            _ => {
                data = format!("{} {} {} - {}\n", &record.now, record.level, record.module_path, record.args);
            }
        }
        self.file.borrow_mut().write(data.as_bytes());
        self.file.borrow_mut().flush();
        None
    }
}