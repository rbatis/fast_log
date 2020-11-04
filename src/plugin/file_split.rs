use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::Path;
use std::sync::RwLock;

use chrono::{DateTime, Local};
use log::Level;

use crate::fast_log::{FastLogRecord, LogAppender};

/// only write append into file
pub struct FileSplitAppender {
    split_log_num: u64,
    temp_log_num: u64,
    create_num: u64,
    dir_path: String,
    file: File,
}

impl FileSplitAppender {
    ///split_log_num: number of log file data num
    ///dir_path the dir
    pub fn new(dir_path: &str, split_log_num: u64) -> FileSplitAppender {
        if !dir_path.is_empty() && dir_path.ends_with(".log") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() && !dir_path.ends_with("/") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() {
            DirBuilder::new().create(dir_path);
        }
        let mut last = open_last_num(dir_path).unwrap();
        last = last + 1;
        let first_file_path = format!("{}{}.log", dir_path.to_string(), last);
        Self {
            split_log_num: split_log_num,
            temp_log_num: 0,
            create_num: last,
            dir_path: dir_path.to_string(),
            file: OpenOptions::new()
                .create(true)
                .append(true)
                .open(first_file_path.as_str())
                .unwrap_or(File::create(Path::new(first_file_path.as_str())).unwrap()),
        }
    }
}

impl LogAppender for FileSplitAppender {
    fn do_log(&mut self, record: &FastLogRecord) {
        let mut data = String::new();
        match record.level {
            Level::Warn | Level::Error => {
                data = format!("{} {} {} - {}  {}\n", &record.now, record.level, record.module_path, record.args, record.format_line());
            }
            _ => {
                data = format!("{} {} {} - {}\n", &record.now, record.level, record.module_path, record.args);
            }
        }
        if self.temp_log_num >= self.split_log_num {
            self.create_num += 1;
            let first_file_path = format!("{}{}.log", self.dir_path.to_string(), self.create_num);
            let create = OpenOptions::new()
                .create(true)
                .append(true)
                .open(first_file_path.as_str());
            if create.is_ok() {
                self.file = create.unwrap();
                write_last_num(&self.dir_path, self.create_num);
                self.temp_log_num = 0;
            } else {
                self.create_num -= 1;
            }
        }
        self.file.write(data.as_bytes());
        self.file.flush();
        self.temp_log_num += 1;
    }
}

fn open_last_num(dir_path: &str) -> Result<u64, String> {
    let mut config = OpenOptions::new()
        .read(true)
        .open(Path::new(format!("{}.fast_log_split_appender", dir_path).as_str()));
    if config.is_err() {
        config = File::create(Path::new(format!("{}.fast_log_split_appender", dir_path).as_str()));
    }
    match config {
        Ok(mut ok) => {
            let mut data = String::new();
            ok.read_to_string(&mut data);
            println!("data:{}", &data);
            let mut last = 0;
            if !data.is_empty() {
                last = data.parse::<u64>().unwrap();
            }
            Ok(last)
        }
        e => {
            return Err(e.err().unwrap().to_string());
        }
    }
}

fn write_last_num(dir_path: &str, last: u64) {
    let mut config = OpenOptions::new()
        .write(true)
        .open(Path::new(format!("{}.fast_log_split_appender", dir_path).as_str()))
        .unwrap();
    config.write(last.to_string().as_bytes());
    config.flush();
}