use crate::appender::{FastLogRecord, LogAppender};
use crate::error::LogError;
use memmap::MmapMut;
use std::cell::{RefCell, UnsafeCell};
use std::fs::{File, OpenOptions};

/// only write append into file
pub struct FileMmapAppender {
    file: RefCell<File>,
    bytes: UnsafeCell<MmapMut>,
}

impl FileMmapAppender {
    pub fn new(log_file_path: &str, size: usize) -> Result<Self, LogError> {
        let log_file_path = log_file_path.replace("\\", "/");
        if let Some(right) = log_file_path.rfind("/") {
            let path = &log_file_path[0..right];
            std::fs::create_dir_all(path);
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)?;
        file.set_len(size as u64);
        let mmap = unsafe {
            memmap::MmapOptions::new()
                .map(&file)
                .map_err(|e| LogError::from(format!("{}", e.to_string())))?
        };
        Ok(Self {
            file: RefCell::new(file),
            bytes: UnsafeCell::new(mmap.make_mut()?),
        })
    }
}

impl LogAppender for FileMmapAppender {
    fn do_logs(&self, records: &[FastLogRecord]) {
        todo!()
    }
}
