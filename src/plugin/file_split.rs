use crate::appender::{Command, FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::{chan, Receiver, Sender};
use fastdate::DateTime;
use std::cell::RefCell;
use std::fs::{DirEntry, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::str::FromStr;
use std::time::Duration;

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

/// split log file allow compress log
pub struct FileSplitAppender {
    pub cell: RefCell<FileSplitAppenderData>,
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
        let paths = std::fs::read_dir(dir);
        if let Ok(paths) = paths {
            let mut paths_vec = vec![];
            for path in paths {
                match path {
                    Ok(path) => {
                        if let Some(v) = path.file_name().to_str() {
                            //filter temp.log and not start with temp
                            if (v.ends_with(".log")
                                && v.trim_end_matches(".log").ends_with(temp_name))
                                || !v.starts_with(temp_name)
                            {
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
                let now = fastdate::DateTime::now();
                for index in 0..paths_vec.len() {
                    let item = &paths_vec[index];
                    let file_name = item.file_name();
                    let name = file_name.to_str().unwrap_or("").to_string();
                    if let Some(time) = self.file_name_parse_time(&name, temp_name) {
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

    fn file_name_parse_time(&self, name: &str, temp_name: &str) -> Option<fastdate::DateTime> {
        if name.starts_with(temp_name) {
            let mut time_str = name.replace(temp_name, "");
            if let Some(v) = time_str.find(".") {
                time_str = time_str[0..v].to_string();
            }
            let time = fastdate::DateTime::from_str(&time_str);
            if let Ok(time) = time {
                return Some(time);
            }
        }
        return None;
    }
}

/// split log file allow pack compress log
/// Memory space swop running time , reduces the number of repeated queries for IO
pub struct FileSplitAppenderData {
    dir_path: String,
    file: File,
    sender: Sender<LogPack>,
    temp_size: LogSize,
    rolling_type: RollingType,
    //cache data
    temp_bytes: usize,
    temp_name: String,
}

impl FileSplitAppenderData {
    /// send data make an pack,and truncate data when finish.
    pub fn send_pack(&mut self) {
        let first_file_path = format!("{}{}.log", self.dir_path, &self.temp_name);
        let new_log_name = format!(
            "{}{}{}.log",
            self.dir_path,
            &self.temp_name,
            DateTime::now()
                .to_string()
                .replace(" ", "T")
                .replace(":", "-")
        );
        std::fs::copy(&first_file_path, &new_log_name);
        self.sender.send(LogPack {
            dir: self.dir_path.clone(),
            rolling: self.rolling_type.clone(),
            new_log_name: new_log_name,
        });
        self.truncate();
    }

    pub fn truncate(&mut self) {
        //reset data
        self.file.set_len(0);
        self.file.seek(SeekFrom::Start(0));
        self.temp_bytes = 0;
    }
}

impl FileSplitAppender {
    pub fn new(
        file_path: &str,
        temp_size: LogSize,
        rolling_type: RollingType,
        packer: Box<dyn Packer>,
    ) -> FileSplitAppender {
        let mut dir_path = file_path.to_owned();
        let mut temp_file_name = dir_path.to_string();
        if dir_path.contains("/") {
            let new_dir_path =
                dir_path[0..dir_path.rfind("/").unwrap_or_default()].to_string() + "/";
            std::fs::create_dir_all(&new_dir_path);
            temp_file_name = dir_path.trim_start_matches(&new_dir_path).to_string();
            dir_path = new_dir_path;
        }
        if temp_file_name.is_empty() {
            temp_file_name = "temp.log".to_string();
        }
        if !dir_path.is_empty() && dir_path.ends_with(".log") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() && !dir_path.ends_with("/") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() {
            std::fs::create_dir_all(&dir_path);
        }
        let file_name = temp_file_name.trim_end_matches(".log");
        let first_file_path = format!("{}{}.log", &dir_path, file_name);
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(first_file_path.as_str());
        if file.is_err() {
            panic!(
                "[fast_log] open and create file fail:{}",
                file.err().unwrap()
            );
        }
        let mut file = file.unwrap();
        let mut temp_bytes = 0;
        if let Ok(m) = file.metadata() {
            temp_bytes = m.len() as usize;
        }
        file.seek(SeekFrom::Start(temp_bytes as u64));
        let (sender, receiver) = chan(None);
        spawn_saver(file_name, receiver, packer);
        Self {
            cell: RefCell::new(FileSplitAppenderData {
                temp_bytes: temp_bytes,
                dir_path: dir_path.to_string(),
                file: file,
                sender: sender,
                temp_size: temp_size,
                temp_name: file_name.to_string(),
                rolling_type: rolling_type,
            }),
        }
    }
}

impl LogAppender for FileSplitAppender {
    fn do_logs(&self, records: &[FastLogRecord]) {
        let mut data = self.cell.borrow_mut();
        //if temp_bytes is full,must send pack
        let mut temp = String::with_capacity(records.len() * 10);
        for x in records {
            match x.command {
                Command::CommandRecord => {
                    if (data.temp_bytes + temp.as_bytes().len() + x.formated.as_bytes().len())
                        > data.temp_size.get_len()
                    {
                        data.temp_bytes += {
                            let w = data.file.write(temp.as_bytes());
                            if let Ok(w) = w {
                                w
                            } else {
                                0
                            }
                        };
                        data.send_pack();
                        temp.clear();
                    }
                    temp.push_str(x.formated.as_str());
                }
                Command::CommandExit => {}
                Command::CommandFlush(_) => {}
            }
        }
        if !temp.is_empty() {
            data.temp_bytes += {
                let w = data.file.write(temp.as_bytes());
                if let Ok(w) = w {
                    w
                } else {
                    0
                }
            };
        }
    }

    fn flush(&self) {
        let mut data = self.cell.borrow_mut();
        data.file.flush();
    }
}

///spawn an saver thread to save log file or zip file
fn spawn_saver(temp_name: &str, r: Receiver<LogPack>, packer: Box<dyn Packer>) {
    let temp = temp_name.to_string();
    std::thread::spawn(move || {
        loop {
            if let Ok(pack) = r.recv() {
                //do rolling
                pack.rolling.do_rolling(&temp, &pack.dir);
                let log_file_path = pack.new_log_name.clone();
                //do save pack
                let remove = do_pack(&packer, pack);
                if let Ok(remove) = remove {
                    if remove {
                        std::fs::remove_file(log_file_path);
                    }
                }
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
    let log_file = log_file.unwrap();
    //make
    let r = packer.do_pack(log_file, log_file_path);
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
