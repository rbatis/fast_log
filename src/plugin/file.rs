use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;

use crate::fast_log::{FastLogRecord, LogAppender};


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
    fn do_log(&self, record: &FastLogRecord){
        self.file.borrow_mut().write(record.formated.as_bytes());
        self.file.borrow_mut().flush();
    }
}