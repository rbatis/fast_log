use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::RwLock;

use log::Level;

use crate::fast_log::LogAppender;

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
    fn do_log(&mut self, info: &str) {
        self.file.write(info.as_bytes());
        self.file.flush();
    }
}