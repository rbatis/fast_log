use crate::appender::{FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::plugin::file_split::{FileSplitAppender, HowToPackType, KeepType, RawFile};
use crate::plugin::packer::LogPacker;

/// Single logs are stored in rolling mode by capacity
pub struct FileLoopAppender {
    file: FileSplitAppender,
}

impl FileLoopAppender {
    pub fn new(log_file_path: &str, size: LogSize) -> Result<FileLoopAppender, LogError> {
        Ok(Self {
            file: FileSplitAppender::new::<RawFile>(
                log_file_path,
                Box::new(HowToPackType::BySize(size)),
                Box::new(KeepType::KeepNum(1)),
                Box::new(LogPacker {}),
            )?,
        })
    }
}

impl LogAppender for FileLoopAppender {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        self.file.do_logs(records);
    }
}
