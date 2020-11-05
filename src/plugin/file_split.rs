use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::Path;
use std::sync::RwLock;

use chrono::{DateTime, Local, SecondsFormat};
use log::Level;

use crate::fast_log::{FastLogRecord, LogAppender};
use zip::write::FileOptions;

/// split log file
pub struct FileSplitAppender {
    split_log_num: u64,
    temp_log_num: u64,
    create_num: u64,
    dir_path: String,
    file: File,
    zip_compress: bool,
}

impl FileSplitAppender {
    ///split_log_num: number of log file data num
    ///dir_path the dir
    pub fn new(dir_path: &str, split_log_num: u64, zip_compress: bool) -> FileSplitAppender {
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
        if zip_compress {
            spawn_to_zip(&format!("{}{}.log", dir_path.to_string(), last));
        }
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
            zip_compress,
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
            let current_file_path = format!("{}{}.log", self.dir_path.to_string(), self.create_num);
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
                if self.zip_compress {
                    //to zip
                    spawn_to_zip(&current_file_path);
                }
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

fn spawn_to_zip(log_file: &str) {
    let log_file = log_file.to_owned();
    std::thread::spawn(move || {
        to_zip(log_file.as_str());
    });
}

fn to_zip(log_file_path: &str) {
    if log_file_path.is_empty() {
        return;
    }
    let log_names: Vec<&str> = log_file_path.split("/").collect();
    let log_name = log_names[log_names.len() - 1];


    let mut log_file = OpenOptions::new()
        .read(true)
        .open(Path::new(log_file_path));
    match log_file {
        Ok(_) => {
            //make zip
            let date=Local::now();
            let date=date.format("%Y_%m_%dT%H_%M_%S").to_string();
            let zip_path = log_file_path.replace(".log", &format!("_{}.zip",date));
            let zip_file = std::fs::File::create(&zip_path);
            match zip_file {
                Ok(zip_file) => {
                    let mut zip = zip::ZipWriter::new(zip_file);
                    zip.start_file(log_name, FileOptions::default());
                    let finish = zip.finish();
                    match finish {
                        Ok(f) => {
                            std::fs::remove_file(log_file_path);
                        }
                        Err(e)  => {
                            //nothing
                            println!("[fast_log] try zip fail{:?}",e);
                        }
                    }
                }
                Err(e) => {
                    println!("[fast_log] create(&{}) fail:{}",zip_path,e);
                }
            }
        }
        Err(e) => {
            println!("[fast_log] give up compress log file. because: {}",e);
        }
    }
}