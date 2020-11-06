use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use log::Level;

use crate::fast_log::{LogAppender, FastLogRecord};

/// only write append into file
pub struct FileAppender {
    file: File
}

impl FileAppender {
    pub fn new(log_file_path: &str) -> FileAppender {
        Self {
            file: OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file_path)
                .unwrap_or(File::create(Path::new(log_file_path)).unwrap())
        }
    }
}

impl LogAppender for FileAppender {
    fn do_log(&mut self, record: &FastLogRecord) {
        let mut data = String::new();
        match record.level {
            Level::Warn | Level::Error => {
                data = format!("{} {} {} - {}  {}\n", &record.now, record.level, record.module_path, record.args, record.format_line());
            }
            _ => {
                data = format!("{} {} {} - {}\n", &record.now, record.level, record.module_path, record.args);
            }
        }
        self.file.write(data.as_bytes());
        self.file.flush();
    }
}