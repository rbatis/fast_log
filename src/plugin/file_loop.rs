use crate::appender::{FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::plugin::file_split::{FileSplitAppender, RollingType};
use crate::plugin::packer::LogPacker;

/// Single logs are stored in rolling mode by capacity
pub struct FileLoopAppender {
    file: FileSplitAppender,
}

impl FileLoopAppender {
    pub fn new(log_file_path: &str, size: LogSize) -> FileLoopAppender {
        Self {
            file: FileSplitAppender::new(
                log_file_path,
                size,
                RollingType::KeepNum(1),
                Box::new(LogPacker {}),
            ),
        }
    }
}

impl LogAppender for FileLoopAppender {
    fn do_logs(&self, records: &[FastLogRecord]) {
        self.file.do_logs(records);
    }

    fn flush(&self) {
        self.file.flush();
    }
}
