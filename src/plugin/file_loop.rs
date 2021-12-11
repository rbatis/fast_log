use crate::appender::{FastLogRecord, LogAppender};
use std::cell::RefCell;
use std::fs::{File, OpenOptions, DirBuilder};
use std::io::Write;
use crate::consts::LogSize;
use crate::plugin::file_split::{FileSplitAppender, RollingType};
use crate::plugin::packer::LogPacker;

/// Single logs are stored in rolling mode by capacity
pub struct FileLoopAppender {
    file: FileSplitAppender,
}

impl FileLoopAppender {
    pub fn new(log_file_path: &str, max_temp_size: LogSize) -> FileLoopAppender {
        let mut path = String::new();
        let mut file = log_file_path.to_string();
        if log_file_path.contains("/"){
            path = log_file_path[0..log_file_path.rfind("/").unwrap_or_default()].to_string()+"/";
            std::fs::create_dir_all(&path);
            file = log_file_path.trim_start_matches(&path).to_string();
        }
        Self {
            file: FileSplitAppender::new(&path, &file, max_temp_size, RollingType::KeepNum(0), Box::new(LogPacker {}))
        }
    }
}

impl LogAppender for FileLoopAppender {
    fn do_log(&self, record: &mut FastLogRecord) {
        self.file.do_log(record);
    }
}
