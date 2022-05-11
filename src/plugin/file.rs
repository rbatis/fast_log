use crate::appender::{FastLogRecord, LogAppender};
use std::cell::RefCell;
use std::fs::{File, OpenOptions, DirBuilder};
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
    fn do_log(&self, record: &FastLogRecord) {
        let mut log_file = self.file.borrow_mut();
        let mut buf = vec![];
        for x in record.formated.bytes() {
            buf.push(x);
        }
        log_file.write_all(buf.as_slice());
    }

    fn flush(&self) {
        self.file.borrow_mut().flush();
    }
}
