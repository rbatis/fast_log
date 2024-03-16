use crate::appender::{Command, FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::plugin::file_name::FileName;
use crate::plugin::file_split::{Keep, LogPack, Packer, SplitFile};
use crate::{chan, Receiver, Sender};
use std::cell::RefCell;
use std::io::SeekFrom;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// rotate log file allow pack compress log
/// Memory space swop running time, reduces the number of repeated queries for IO
pub struct FileRotateAppender<F: SplitFile, R: Rotate> {
    dir_path: String,
    file: RefCell<F>,
    sender: Sender<LogPack>,
    temp_size: LogSize,
    // temp data length
    temp_bytes: AtomicUsize,
    rolling_type: Arc<R>,
}

impl<F, R> FileRotateAppender<F, R>
where
    F: SplitFile,
    R: Keep + Rotate + 'static,
{
    pub fn new(
        file_path: &str,
        temp_size: LogSize,
        rolling_type: R,
        packer: Box<dyn Packer>,
    ) -> Result<FileRotateAppender<F, R>, LogError> {
        let mut dir_path = file_path
            .trim_end_matches(&file_path.extract_file_name())
            .to_string();
        if dir_path.is_empty() {
            if let Ok(v) = std::env::current_dir() {
                dir_path = v.to_str().unwrap_or_default().to_string();
            }
        }
        let _ = std::fs::create_dir_all(&dir_path);
        let path = Path::new(&dir_path);
        let temp_file = path.join(rolling_type.init(&dir_path, &packer));
        let temp_bytes = AtomicUsize::new(0);
        let file = F::new(temp_file.to_str().unwrap(), temp_size)?;
        let mut offset = file.offset();
        if offset != 0 {
            offset += 1;
        }
        temp_bytes.store(offset, Ordering::Relaxed);
        let _ = file.seek(SeekFrom::Start(temp_bytes.load(Ordering::Relaxed) as u64));
        let (sender, receiver) = chan(None);
        let arc_rolling_type = Arc::new(rolling_type);
        let arc_packer = Arc::new(packer);
        spawn_saver(
            arc_rolling_type.base_name().to_string(),
            receiver,
            arc_rolling_type.clone(),
            arc_packer.clone(),
        );
        Ok(Self {
            temp_bytes,
            dir_path: dir_path.to_string(),
            file: RefCell::new(file),
            sender,
            temp_size,
            rolling_type: arc_rolling_type.clone(),
        })
    }

    /// send data truncate data, and make an pack.
    fn send_pack(&self, record: &FastLogRecord) {
        let current_filename = Path::new(&self.dir_path).join(self.rolling_type.current());
        self.rolling_type.next(record);
        self.truncate();

        let _ = self.sender.send(LogPack {
            dir: self.dir_path.clone(),
            new_log_name: current_filename.to_str().unwrap().to_string(),
            wg: None,
        });
    }

    pub fn truncate(&self) {
        // flush data
        self.file.borrow().flush();
        // create new file
        let temp_file = Path::new(&self.dir_path).join(self.rolling_type.current());
        match F::new(temp_file.to_str().unwrap(), self.temp_size) {
            Ok(f) => {
                *self.file.borrow_mut() = f;
                self.temp_bytes.store(0, Ordering::SeqCst);
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

impl<F, R> LogAppender for FileRotateAppender<F, R>
where
    F: SplitFile,
    R: Keep + Rotate + Sync + 'static,
{
    fn do_logs(&self, records: &[FastLogRecord]) {
        //if temp_bytes is full or rotate condition satisfied, must send pack
        let mut temp = String::with_capacity(records.len() * 10);
        for x in records {
            match x.command {
                Command::CommandRecord => {
                    let should_rotate = self.rolling_type.should_rotate(x);
                    if (self.temp_bytes.load(Ordering::Relaxed)
                        + temp.as_bytes().len()
                        + x.formated.as_bytes().len())
                        >= self.temp_size.get_len()
                        || should_rotate
                    {
                        self.temp_bytes.fetch_add(
                            {
                                let w = self.file.borrow().write(temp.as_bytes());
                                if let Ok(w) = w {
                                    w
                                } else {
                                    0
                                }
                            },
                            Ordering::SeqCst,
                        );
                        temp.clear();
                        self.send_pack(x);
                    }
                    temp.push_str(x.formated.as_str());
                }
                Command::CommandExit => {}
                Command::CommandFlush(ref w) => {
                    let _ = self.sender.send(LogPack {
                        dir: "".to_string(),
                        new_log_name: "".to_string(),
                        wg: Some(w.clone()),
                    });
                }
            }
        }
        if !temp.is_empty() {
            self.temp_bytes.fetch_add(
                {
                    let w = self.file.borrow().write(temp.as_bytes());
                    if let Ok(w) = w {
                        w
                    } else {
                        0
                    }
                },
                Ordering::SeqCst,
            );
        }
    }
}

pub trait Rotate: Send + Sync {
    /// base file name
    fn base_name(&self) -> &str {
        "log.log"
    }

    /// check exist log files, return last file name for write
    fn init(&self, dir_path: &str, packer: &Box<dyn Packer>) -> String;

    /// current file name
    fn current(&self) -> String;

    /// next file name
    fn next(&self, record: &FastLogRecord) -> String;

    /// check if should rotate
    fn should_rotate(&self, _record: &FastLogRecord) -> bool {
        false
    }
}

///spawn an saver thread to save log file or zip file
fn spawn_saver<R: Keep + Sync + 'static>(
    temp_name: String,
    r: Receiver<LogPack>,
    rolling_type: Arc<R>,
    packer: Arc<Box<dyn Packer>>,
) {
    std::thread::spawn(move || {
        loop {
            if let Ok(pack) = r.recv() {
                if pack.wg.is_some() {
                    return;
                }
                let log_file_path = pack.new_log_name.clone();
                //do save pack
                let remove = pack.do_pack(packer.as_ref());
                if let Ok(remove) = remove {
                    if remove {
                        let _ = std::fs::remove_file(log_file_path);
                    }
                }
                //do rolling
                rolling_type.do_keep(&pack.dir, &temp_name);
            } else {
                break;
            }
        }
    });
}
