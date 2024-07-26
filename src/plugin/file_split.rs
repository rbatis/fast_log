use crate::appender::{Command, FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::plugin::file_name::FileName;
use crate::{chan, Receiver, Sender, WaitGroup};
use fastdate::DateTime;
use std::cell::RefCell;
use std::fs::{DirEntry, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// .zip or .lz4 or any one packer
///
/// must impl pack_name,is_allow,do_pack method
pub trait Packer: Send + Sync {
    fn pack_name(&self) -> &'static str;

    ///return bool: remove_log_file
    fn do_pack(&self, log_file: File, log_file_path: &str) -> Result<bool, LogError>;

    /// default 0 is not retry pack. if retry > 0 ,it will trying rePack
    fn retry(&self) -> i32 {
        return 0;
    }

    /// date to string
    fn date_to_string(&self, arg: DateTime) -> String {
        arg.display_stand()
            .to_string()
            .replace(" ", "T")
            .replace(":", "-")
    }

    /// create date style log name
    /// input: 'temp.log'
    /// output: 'temp2024-07-26T16-04-17.685429.log'
    fn new_data_log_name(&self, first_file_path: &str, date: DateTime) -> String {
        let file_name = first_file_path.extract_file_name();
        let mut new_log_name = String::new();
        let point = file_name.rfind(".");
        match point {
            None => {
                new_log_name.push_str(&self.date_to_string(date));
            }
            Some(i) => {
                let (name, ext) = file_name.split_at(i);
                new_log_name = format!("{}{}{}", name, self.date_to_string(date), ext);
            }
        }
        new_log_name = first_file_path.trim_end_matches(&file_name).to_string() + &new_log_name;
        return new_log_name;
    }
}

/// is can do pack?
pub trait CanPack: Send {
    fn is(&mut self, temp_size: usize, arg: &FastLogRecord) -> bool;
}

/// keep logs, for example keep by log num or keep by log create time.
/// that do not meet the retention conditions will be deleted
/// you can use KeepType or RollingType::All
pub trait Keep: Send {
    /// return removed nums
    fn do_keep(&self, dir: &str, temp_name: &str) -> i64;
    fn read_paths(&self, dir: &str, temp_name: &str) -> Vec<DirEntry> {
        let base_name = get_base_name(temp_name);
        let paths = std::fs::read_dir(dir);
        if let Ok(paths) = paths {
            //let mut temp_file = None;
            let mut paths_vec = vec![];
            for path in paths {
                match path {
                    Ok(path) => {
                        if let Some(v) = path.file_name().to_str() {
                            if v == temp_name {
                                continue;
                            }
                            if !v.starts_with(&base_name) {
                                continue;
                            }
                        }
                        paths_vec.push(path);
                    }
                    _ => {}
                }
            }
            paths_vec.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            return paths_vec;
        }
        return vec![];
    }
}

pub trait SplitFile: Send {
    fn new(path: &str) -> Result<Self, LogError>
    where
        Self: Sized;
    fn seek(&self, pos: SeekFrom) -> std::io::Result<u64>;
    fn write(&self, buf: &[u8]) -> std::io::Result<usize>;
    fn truncate(&self) -> std::io::Result<()>;
    fn flush(&self);
    fn len(&self) -> usize;
    fn offset(&self) -> usize;
}


///only use File
pub struct RawFile {
    pub inner: RefCell<File>,
}

impl From<File> for RawFile {
    fn from(value: File) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
}

impl SplitFile for RawFile {
    fn new(path: &str) -> Result<Self, LogError>
    where
        Self: Sized,
    {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)?;
        Ok(Self {
            inner: RefCell::new(file),
        })
    }

    fn seek(&self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.borrow_mut().seek(pos)
    }

    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }

    fn truncate(&self) -> std::io::Result<()> {
        self.inner.borrow_mut().set_len(0)?;
        self.inner.borrow_mut().flush()?;
        self.inner.borrow_mut().seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn flush(&self) {
        let _ = self.inner.borrow_mut().flush();
    }

    fn len(&self) -> usize {
        if let Ok(v) = self.inner.borrow_mut().metadata() {
            v.len() as usize
        } else {
            0
        }
    }

    fn offset(&self) -> usize {
        let mut offset = self.len();
        if offset > 0 {
            offset = offset - 1;
        }
        offset
    }
}


pub enum PackType {
    ByDate(DateTime),
    BySize(LogSize),
}

impl CanPack for PackType {
    fn is(&mut self, temp_size: usize, arg: &FastLogRecord) -> bool {
        return match self {
            PackType::ByDate(date_time) => {
                let dt = fastdate::DateTime::from_system_time(arg.now, fastdate::offset_sec());
                if dt.day() > date_time.day() {
                    *date_time = dt;
                    true
                } else {
                    false
                }
            }
            PackType::BySize(limit) => {
                if temp_size >= limit.get_len() {
                    true
                } else {
                    false
                }
            }
        };
    }
}

/// split log file allow pack compress log
/// Memory space swop running time , reduces the number of repeated queries for IO
pub struct FileSplitAppender {
    file: Box<dyn SplitFile>,
    packer: Arc<Box<dyn Packer>>,
    dir_path: String,
    sender: Sender<LogPack>,
    is_pack: Box<dyn CanPack>,
    //cache data
    temp_bytes: AtomicUsize,
    temp_name: String,
}

impl FileSplitAppender {
    pub fn new<F: SplitFile + 'static>(
        file_path: &str,
        how_to_pack: Box<dyn CanPack>,
        rolling_type: Box<dyn Keep>,
        packer: Box<dyn Packer>,
    ) -> Result<FileSplitAppender, LogError> {
        let temp_name = {
            let mut name = file_path.extract_file_name().to_string();
            if name.is_empty() {
                name = "temp.log".to_string();
            }
            name
        };
        let mut dir_path = file_path.trim_end_matches(&temp_name).to_string();
        if dir_path.is_empty() {
            if let Ok(v) = std::env::current_dir() {
                dir_path = v.to_str().unwrap_or_default().to_string();
            }
        }
        let _ = std::fs::create_dir_all(&dir_path);
        let mut sp = "";
        if !dir_path.is_empty() {
            sp = "/";
        }
        let temp_file = format!("{}{}{}", dir_path, sp, temp_name);
        let temp_bytes = AtomicUsize::new(0);
        let file = F::new(&temp_file)?;
        let mut offset = file.offset();
        if offset != 0 {
            offset += 1;
        }
        temp_bytes.store(offset, Ordering::Relaxed);
        let _ = file.seek(SeekFrom::Start(temp_bytes.load(Ordering::Relaxed) as u64));
        let (sender, receiver) = chan(None);
        let arc_packer = Arc::new(packer);
        spawn_saver(
            temp_name.clone(),
            receiver,
            rolling_type,
            arc_packer.clone(),
        );
        Ok(Self {
            temp_bytes,
            dir_path: dir_path.to_string(),
            file: Box::new(file) as Box<dyn SplitFile>,
            sender,
            is_pack: how_to_pack,
            temp_name,
            packer: arc_packer,
        })
    }
    /// send data make an pack,and truncate data when finish.
    pub fn send_pack(&self, time: SystemTime, wg: Option<WaitGroup>) {
        let mut sp = "";
        if !self.dir_path.is_empty() && !self.dir_path.ends_with("/") {
            sp = "/";
        }
        let first_file_path = format!("{}{}{}", self.dir_path, sp, &self.temp_name);
        let date = DateTime::from_system_time(time, fastdate::offset_sec());
        let new_log_name = self
            .packer
            .new_data_log_name(&first_file_path, date);
        self.file.flush();
        let _ = std::fs::copy(&first_file_path, &new_log_name);
        let _ = self.sender.send(LogPack {
            dir: self.dir_path.clone(),
            new_log_name: new_log_name,
            wg: wg,
        });
        self.truncate();
    }

    pub fn truncate(&self) {
        //reset data
        let _ = self.file.truncate();
        self.temp_bytes.store(0, Ordering::SeqCst);
    }
}

///log data pack
pub struct LogPack {
    pub dir: String,
    pub new_log_name: String,
    pub wg: Option<WaitGroup>,
}

impl LogPack {
    /// write an Pack to zip file
    pub fn do_pack(&self, packer: &Box<dyn Packer>) -> Result<bool, LogError> {
        let log_file_path = self.new_log_name.as_str();
        if log_file_path.is_empty() {
            return Err(LogError::from("log_file_path.is_empty"));
        }
        let log_file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(log_file_path);
        if log_file.is_err() {
            return Err(LogError::from(format!(
                "open(log_file_path={}) fail",
                log_file_path
            )));
        }
        //make
        let r = packer.do_pack(log_file.unwrap(), log_file_path);
        if r.is_err() && packer.retry() > 0 {
            let mut retry = 1;
            while let Err(_packs) = self.do_pack(packer) {
                retry += 1;
                if retry > packer.retry() {
                    break;
                }
            }
        }
        if let Ok(b) = r {
            return Ok(b);
        }
        return Ok(false);
    }
}


///rolling keep type
pub type RollingType = KeepType;

///rolling keep type
#[derive(Copy, Clone, Debug)]
pub enum KeepType {
    /// keep All of log packs
    All,
    /// keep by Time Duration,
    /// for example:
    /// // keep one day log pack
    /// (Duration::from_secs(24 * 3600))
    KeepTime(Duration),
    /// keep log pack num(.log,.zip.lz4...more)
    KeepNum(i64),
    KeepDate,
}

impl Keep for KeepType {
    fn do_keep(&self, dir: &str, temp_name: &str) -> i64 {
        let mut removed = 0;
        match self {
            KeepType::KeepNum(n) => {
                let paths_vec = self.read_paths(dir, temp_name);
                for index in 0..paths_vec.len() {
                    if index >= (*n) as usize {
                        let item = &paths_vec[index];
                        let _ = std::fs::remove_file(item.path());
                        removed += 1;
                    }
                }
            }
            KeepType::KeepTime(duration) => {
                let paths_vec = self.read_paths(dir, temp_name);
                let now = DateTime::now();
                for index in 0..paths_vec.len() {
                    let item = &paths_vec[index];
                    if let Ok(m) = item.metadata() {
                        if let Ok(c) = m.created() {
                            let time = DateTime::from(c);
                            if now.clone().sub(duration.clone()) > time {
                                let _ = std::fs::remove_file(item.path());
                                removed += 1;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        removed
    }
}

impl LogAppender for FileSplitAppender {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        //if temp_bytes is full,must send pack
        let mut temp = String::with_capacity(records.len() * 10);
        for x in records {
            match x.command {
                Command::CommandRecord => {
                    let current_temp_size = self.temp_bytes.load(Ordering::Relaxed)
                        + temp.as_bytes().len()
                        + x.formated.as_bytes().len();
                    if self.is_pack.is(current_temp_size, x) {
                        self.temp_bytes.fetch_add(
                            {
                                let w = self.file.write(temp.as_bytes());
                                if let Ok(w) = w {
                                    w
                                } else {
                                    0
                                }
                            },
                            Ordering::SeqCst,
                        );
                        temp.clear();
                        self.send_pack(x.now, None);
                    }
                    temp.push_str(x.formated.as_str());
                }
                Command::CommandExit => {}
                Command::CommandFlush(ref w) => {
                    let current_temp_size = self.temp_bytes.load(Ordering::Relaxed);
                    if self.is_pack.is(current_temp_size, x) {
                        self.temp_bytes.fetch_add(
                            {
                                let w = self.file.write(temp.as_bytes());
                                if let Ok(w) = w {
                                    w
                                } else {
                                    0
                                }
                            },
                            Ordering::SeqCst,
                        );
                        temp.clear();
                        self.send_pack(x.now, Some(w.clone()));
                    }
                }
            }
        }
        if !temp.is_empty() {
            let _ = self.temp_bytes.fetch_add(
                {
                    let w = self.file.write(temp.as_bytes());
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

///spawn an saver thread to save log file or zip file
fn spawn_saver(
    temp_name: String,
    r: Receiver<LogPack>,
    rolling_type: Box<dyn Keep>,
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

fn get_base_name(path: &str) -> String {
    let file_name = path.extract_file_name();
    let p = file_name.rfind(".");
    match p {
        None => file_name,
        Some(i) => file_name[0..i].to_string(),
    }
}
