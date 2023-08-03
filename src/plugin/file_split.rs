use crate::appender::{Command, FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::{chan, Receiver, Sender};
use fastdate::DateTime;
use std::cell::RefCell;
use std::fs::{DirEntry, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub trait SplitFile: Send {
    fn new(path: &str, temp_size: LogSize) -> Result<Self, LogError>
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
    fn new(path: &str, temp_size: LogSize) -> Result<Self, LogError>
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
        self.inner.borrow_mut().set_len(0);
        self.inner.borrow_mut().flush();
        self.inner.borrow_mut().seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn flush(&self) {
        self.inner.borrow_mut().flush();
    }

    fn len(&self) -> usize {
        if let Ok(v) = self.inner.borrow_mut().metadata() {
            v.len() as usize
        } else {
            0
        }
    }

    fn offset(&self) -> usize {
        todo!()
    }
}

/// .zip or .lz4 or any one packer
pub trait Packer: Send {
    fn pack_name(&self) -> &'static str;
    //return bool: remove_log_file
    fn do_pack(&self, log_file: File, log_file_path: &str) -> Result<bool, LogError>;
    /// default 0 is not retry pack. if retry > 0 ,it will trying rePack
    fn retry(&self) -> i32 {
        return 0;
    }
}

/// split log file allow pack compress log
/// Memory space swop running time , reduces the number of repeated queries for IO
pub struct FileSplitAppender<F: SplitFile> {
    dir_path: String,
    file: F,
    sender: Sender<LogPack>,
    temp_size: LogSize,
    rolling_type: RollingType,
    //cache data
    temp_bytes: AtomicUsize,
    temp_name: String,
}

impl<F: SplitFile> FileSplitAppender<F> {
    pub fn new(
        file_path: &str,
        temp_size: LogSize,
        rolling_type: RollingType,
        packer: Box<dyn Packer>,
    ) -> Result<FileSplitAppender<F>, LogError> {
        let mut path_buf = PathBuf::from(&file_path);
        if file_path.ends_with("/") || file_path.ends_with("\\") {
            path_buf.push("temp.log");
        }
        let temp_file_name = path_buf
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        let mut dir_path = path_buf.to_str().unwrap_or_default().to_string();
        dir_path = dir_path.trim_end_matches(&temp_file_name).to_string();
        if dir_path.is_empty() {
            if let Ok(v) = std::env::current_dir() {
                dir_path = v.to_str().unwrap_or_default().to_string();
            }
        }
        std::fs::create_dir_all(&dir_path);
        let mut sp = "";
        if !dir_path.is_empty() {
            sp = "/";
        }
        let temp_file = format!("{}{}{}", dir_path, sp, temp_file_name);
        let temp_bytes = AtomicUsize::new(0);
        let file = F::new(&temp_file, temp_size)?;
        let mut offset = file.offset();
        if offset != 0 {
            offset += 1;
        }
        temp_bytes.store(offset, Ordering::Relaxed);
        file.seek(SeekFrom::Start(temp_bytes.load(Ordering::Relaxed) as u64));
        let (sender, receiver) = chan(None);
        spawn_saver(temp_file_name.clone(), receiver, packer);
        Ok(Self {
            temp_bytes,
            dir_path: dir_path.to_string(),
            file,
            sender,
            temp_size,
            temp_name: temp_file_name,
            rolling_type,
        })
    }
    /// send data make an pack,and truncate data when finish.
    pub fn send_pack(&self) {
        let mut sp = "";
        if !self.dir_path.is_empty() && !self.dir_path.ends_with("/") {
            sp = "/";
        }
        let first_file_path = format!("{}{}{}", self.dir_path, sp, &self.temp_name);
        let path = PathBuf::from(first_file_path.clone());
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();

        let mut new_log_name = file_name.to_string();
        let point = file_name.rfind(".");
        match point {
            None => {
                new_log_name.push_str(
                    &DateTime::now()
                        .to_string()
                        .replace(" ", "T")
                        .replace(":", "-"),
                );
            }
            Some(i) => {
                let (name, ext) = file_name.split_at(i);
                new_log_name = format!(
                    "{}{}{}",
                    name,
                    DateTime::now()
                        .to_string()
                        .replace(" ", "T")
                        .replace(":", "-"),
                    ext
                );
            }
        }
        new_log_name = first_file_path.trim_end_matches(&file_name).to_string() + &new_log_name;
        self.file.flush();
        std::fs::copy(&first_file_path, &new_log_name);
        self.sender.send(LogPack {
            dir: self.dir_path.clone(),
            rolling: self.rolling_type.clone(),
            new_log_name: new_log_name,
        });
        self.truncate();
    }

    pub fn truncate(&self) {
        //reset data
        self.file.truncate();
        self.temp_bytes.store(0, Ordering::SeqCst);
    }
}

///log data pack
pub struct LogPack {
    pub dir: String,
    pub rolling: RollingType,
    pub new_log_name: String,
}

///rolling keep type
#[derive(Copy, Clone, Debug)]
pub enum RollingType {
    /// keep All of log packs
    All,
    /// keep by Time Duration,
    /// for example:
    /// // keep one day log pack
    /// (Duration::from_secs(24 * 3600))
    KeepTime(Duration),
    /// keep log pack num(.log,.zip.lz4...more)
    KeepNum(i64),
}

impl RollingType {
    fn read_paths(&self, dir: &str, temp_name: &str) -> Vec<DirEntry> {
        let base_name = get_base_name(&PathBuf::from(temp_name));
        let paths = std::fs::read_dir(dir);
        if let Ok(paths) = paths {
            //let mut temp_file = None;
            let mut paths_vec = vec![];
            for path in paths {
                match path {
                    Ok(path) => {
                        if let Some(v) = path.file_name().to_str() {
                            if v == temp_name {
                                //temp_file = Some(path);
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
            // if let Some(v) = temp_file {
            //     paths_vec.push(v);
            // }
            return paths_vec;
        }
        return vec![];
    }

    pub fn do_rolling(&self, temp_name: &str, dir: &str) -> i64 {
        let mut removed = 0;
        match self {
            RollingType::KeepNum(n) => {
                let paths_vec = self.read_paths(dir, temp_name);
                for index in 0..paths_vec.len() {
                    if index >= (*n) as usize {
                        let item = &paths_vec[index];
                        std::fs::remove_file(item.path());
                        removed += 1;
                    }
                }
            }
            RollingType::KeepTime(duration) => {
                let paths_vec = self.read_paths(dir, temp_name);
                let now = DateTime::now();
                for index in 0..paths_vec.len() {
                    let item = &paths_vec[index];
                    let file_name = item.file_name();
                    let name = file_name.to_str().unwrap_or("").to_string();
                    if let Some(time) = Self::file_name_parse_time(&name, temp_name) {
                        if now.clone().sub(duration.clone()) > time {
                            std::fs::remove_file(item.path());
                            removed += 1;
                        }
                    }
                }
            }
            _ => {}
        }
        removed
    }

    /// parse `temp2023-07-20T10-13-17.452247.log`
    pub fn file_name_parse_time(name: &str, temp_name: &str) -> Option<fastdate::DateTime> {
        let base_name = get_base_name(&PathBuf::from(temp_name));
        if name.starts_with(&base_name) {
            let mut time_str = name.trim_start_matches(&base_name).to_string();
            if let Some(v) = time_str.rfind(".") {
                time_str = time_str[0..v].to_string();
            }
            let time = DateTime::parse("YYYY-MM-DDThh:mm:ss.000000", &time_str);
            if let Ok(time) = time {
                return Some(time);
            }
        }
        return None;
    }
}

impl<F: SplitFile> LogAppender for FileSplitAppender<F> {
    fn do_logs(&self, records: &[FastLogRecord]) {
        //if temp_bytes is full,must send pack
        let mut temp = String::with_capacity(records.len() * 10);
        for x in records {
            match x.command {
                Command::CommandRecord => {
                    if (self.temp_bytes.load(Ordering::Relaxed)
                        + temp.as_bytes().len()
                        + x.formated.as_bytes().len())
                        > self.temp_size.get_len()
                    {
                        self.send_pack();
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
                    }
                    temp.push_str(x.formated.as_str());
                }
                Command::CommandExit => {}
                Command::CommandFlush(_) => {}
            }
        }
        if !temp.is_empty() {
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
        }
    }

    fn flush(&self) {}
}

///spawn an saver thread to save log file or zip file
fn spawn_saver(temp_name: String, r: Receiver<LogPack>, packer: Box<dyn Packer>) {
    std::thread::spawn(move || {
        loop {
            if let Ok(pack) = r.recv() {
                //do rolling
                pack.rolling.do_rolling(&temp_name, &pack.dir);
                let log_file_path = pack.new_log_name.clone();
                //do save pack
                let remove = do_pack(&packer, pack);
                if let Ok(remove) = remove {
                    if remove {
                        std::fs::remove_file(log_file_path);
                    }
                }
            } else {
                break;
            }
        }
    });
}

/// write an Pack to zip file
pub fn do_pack(packer: &Box<dyn Packer>, mut pack: LogPack) -> Result<bool, LogPack> {
    let log_file_path = pack.new_log_name.as_str();
    if log_file_path.is_empty() {
        return Err(pack);
    }
    let log_file = OpenOptions::new().read(true).open(log_file_path);
    if log_file.is_err() {
        return Err(pack);
    }
    //make
    let r = packer.do_pack(log_file.unwrap(), log_file_path);
    if r.is_err() && packer.retry() > 0 {
        let mut retry = 1;
        while let Err(packs) = do_pack(packer, pack) {
            pack = packs;
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

fn get_base_name(path: &PathBuf) -> String {
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    let p = file_name.rfind(".");
    match p {
        None => file_name,
        Some(i) => file_name[0..i].to_string(),
    }
}
