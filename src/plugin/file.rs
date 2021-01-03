use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;
use crate::appender::{FastLogRecord, LogAppender};


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
    fn do_log(&self, record: &FastLogRecord) {
        let mut log_file = self.file.borrow_mut();
        log_file.write(record.formated.as_bytes());
        log_file.flush();
    }
}