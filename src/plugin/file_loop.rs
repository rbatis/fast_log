use crate::appender::{FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::plugin::file_split::{FileSplitAppender, RollingType, SplitFile};
use crate::plugin::packer::LogPacker;

/// Single logs are stored in rolling mode by capacity
pub struct FileLoopAppender<F: SplitFile> {
    file: FileSplitAppender<F>,
}

impl<F: SplitFile> FileLoopAppender<F> {
    pub fn new(log_file_path: &str, size: LogSize) -> Result<FileLoopAppender<F>, LogError> {
        Ok(Self {
            file: FileSplitAppender::<F>::new(
                F::new(log_file_path)?,
                log_file_path,
                size,
                RollingType::KeepNum(1),
                Box::new(LogPacker {}),
            )?,
        })
    }
}

impl<F: SplitFile> LogAppender for FileLoopAppender<F> {
    fn do_logs(&self, records: &[FastLogRecord]) {
        self.file.do_logs(records);
    }

    fn flush(&self) {
        self.file.flush();
    }
}
