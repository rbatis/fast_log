use crate::appender::{FastLogRecord, LogAppender};
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;

/// only write append into file
pub struct FileAppender {
    file: RefCell<File>,
}

impl FileAppender {
    pub fn new(log_file_path: &str) -> FileAppender {
        let log_file_path = log_file_path.replace("\\", "/");
        if let Some(right) = log_file_path.rfind("/") {
            let path = &log_file_path[0..right];
            std::fs::create_dir_all(path);
        }
        Self {
            file: RefCell::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file_path)
                    .unwrap(),
            ),
        }
    }
}

impl LogAppender for FileAppender {
    fn do_logs(&self, records: &[FastLogRecord]) {
        let mut log_file = self.file.borrow_mut();
        let mut buf = String::new();
        for x in records {
            buf.push_str(&x.formated);
        }
        log_file.write_all(buf.as_bytes());
    }

    fn flush(&self) {
        self.file.borrow_mut().flush();
    }
}
