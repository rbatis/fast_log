use crate::appender::{FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::plugin::file_split::SplitFile;
use memmap::MmapMut;
use std::cell::{RefCell, UnsafeCell};
use std::fs::{File, Metadata, OpenOptions};
use std::io::SeekFrom;

/// file size must = temp_size
pub struct MmapFile {
    file: RefCell<File>,
    bytes: UnsafeCell<MmapMut>,
    size: LogSize,
}

impl MmapFile {
    pub fn new(log_file_path: &str, size: LogSize) -> Result<Self, LogError> {
        let log_file_path = log_file_path.replace("\\", "/");
        if let Some(right) = log_file_path.rfind("/") {
            let path = &log_file_path[0..right];
            std::fs::create_dir_all(path);
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)?;
        file.set_len(size.get_len() as u64);
        let mmap = unsafe {
            memmap::MmapOptions::new()
                .map(&file)
                .map_err(|e| LogError::from(format!("{}", e.to_string())))?
        };
        Ok(Self {
            file: RefCell::new(file),
            bytes: UnsafeCell::new(mmap.make_mut()?),
            size,
        })
    }
}

impl SplitFile for MmapFile {
    fn new(path: &str) -> Result<Self, LogError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn seek(&self, pos: SeekFrom) -> std::io::Result<u64> {
        todo!()
    }

    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        todo!()
    }

    fn metadata(&self) -> std::io::Result<Metadata> {
        todo!()
    }
}
