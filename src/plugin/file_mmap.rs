use crate::consts::LogSize;
use crate::error::LogError;
use crate::plugin::file_split::SplitFile;
use memmap::{MmapMut, MmapOptions};
use std::cell::{RefCell, UnsafeCell};
use std::fs::{File, Metadata, OpenOptions};
use std::io::SeekFrom;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicU64, Ordering};

/// file size must = temp_size
pub struct MmapFile {
    file: UnsafeCell<File>,
    bytes: RefCell<MmapMut>,
    size: LogSize,
    point: AtomicU64,
}

impl MmapFile {
    pub fn new(log_file_path: &str, size: LogSize) -> Result<Self, LogError> {
        let log_file_path = log_file_path.replace("\\", "/");
        if let Some(right) = log_file_path.rfind("/") {
            let path = &log_file_path[0..right];
            std::fs::create_dir_all(path);
        }
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&log_file_path)?;
        file.set_len(size.get_len() as u64);
        let mmap = unsafe {
            MmapOptions::new().map(&file).map_err(|e| {
                println!("e={}", e);
                LogError::from(format!("{}", e.to_string()))
            })?
        };
        Ok(Self {
            file: UnsafeCell::new(file),
            bytes: RefCell::new(mmap.make_mut()?),
            size,
            point: AtomicU64::new(0),
        })
    }
}

impl SplitFile for MmapFile {
    fn new(path: &str) -> Result<Self, LogError>
    where
        Self: Sized,
    {
        let mut path = path.to_string();
        if !path.contains("?") {
            path.push_str("?1GB");
        }
        let index = path.rfind("?").unwrap_or_default();
        let file_path = &path[0..index];
        let file_size = &path[(index + 1)..path.len()];
        let mut size = LogSize::parse(file_size)?;
        if size.len() == 0 {
            size = LogSize::GB(1);
        }
        Ok(MmapFile::new(file_path, size)?)
    }

    fn seek(&self, pos: SeekFrom) -> std::io::Result<u64> {
        let len = self.size.len() as u64;
        let new_pos = match pos {
            SeekFrom::Start(n) => {
                if n > len {
                    len
                } else {
                    n
                }
            }
            SeekFrom::End(n) => {
                if n < 0 {
                    let end_offset = len.saturating_add(-n as u64);
                    if end_offset == 0 {
                        0
                    } else {
                        end_offset - 1
                    }
                } else {
                    len.checked_sub(n as u64).ok_or(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek before start of file",
                    ))?
                }
            }
            SeekFrom::Current(n) => {
                let current = self.point.load(Ordering::Relaxed);
                let offset = current.checked_add(n as u64).ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Seek before start of file",
                ))?;
                if offset > len {
                    len
                } else {
                    offset
                }
            }
        };
        self.point.store(new_pos, Ordering::SeqCst);
        Ok(new_pos)
    }

    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        let len = buf.len() as u64;
        let size = self.size.get_len();
        if self.point.load(Ordering::Relaxed) + len > size as u64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Write past end of file",
            ));
        }
        let mut bytes = self.bytes.borrow_mut();
        let current = self.point.load(Ordering::Relaxed) as usize;
        bytes.deref_mut()[current..(current + buf.len())].copy_from_slice(buf);
        let len = buf.len();
        self.point.fetch_add(len as u64, Ordering::SeqCst);
        Ok(len)
    }

    fn metadata(&self) -> std::io::Result<Metadata> {
        unsafe { &*self.file.get() }.metadata()
    }

    fn truncate(&self) -> std::io::Result<()> {
        let file = unsafe { &mut *self.file.get() };
        file.set_len(0)?;
        file.set_len(self.size.get_len() as u64);
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Mmap create fail={}", e.to_string()),
                    )
                })?
                .make_mut()?
        };
        *self.bytes.borrow_mut() = mmap;
        self.point.store(0, Ordering::SeqCst);
        Ok(())
    }

    fn flush(&self) {
        self.bytes.borrow_mut().flush();
    }
}
