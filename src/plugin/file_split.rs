use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use chrono::{Local};
use log::Level;

use crate::fast_log::{FastLogRecord, LogAppender};
use zip::write::FileOptions;
use std::cell::RefCell;


/// split log file allow zip compress log
pub struct FileSplitAppender {
    cell:RefCell<FileSplitAppenderData>
}

/// split log file allow zip compress log
pub struct FileSplitAppenderData {
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
    pub fn new(dir_path: &str, split_log_num: u64, allow_zip_compress: bool) -> FileSplitAppender {
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
        if allow_zip_compress {
            spawn_to_zip(&format!("{}{}.log", dir_path.to_string(), last));
        }
        last = last + 1;
        let first_file_path = format!("{}{}.log", dir_path.to_string(), last);
        Self {
            cell:RefCell::new(FileSplitAppenderData{
                split_log_num: split_log_num,
                temp_log_num: 0,
                create_num: last,
                dir_path: dir_path.to_string(),
                file: OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(first_file_path.as_str())
                    .unwrap_or(File::create(Path::new(first_file_path.as_str())).unwrap()),
                zip_compress: allow_zip_compress,
            })
        }
    }
}

impl LogAppender for FileSplitAppender {
    fn do_log(&self, record: &FastLogRecord) {
        let mut log = String::new();
        match record.level {
            Level::Warn | Level::Error => {
                log = format!("{} {} {} - {}  {}\n", &record.now, record.level, record.module_path, record.args, record.format_line());
            }
            _ => {
                log = format!("{} {} {} - {}\n", &record.now, record.level, record.module_path, record.args);
            }
        }
        let mut data=self.cell.borrow_mut();
        if data.temp_log_num >= data.split_log_num {
            let current_file_path = format!("{}{}.log", data.dir_path.to_string(), data.create_num);
            data.create_num += 1;
            let first_file_path = format!("{}{}.log", data.dir_path.to_string(), data.create_num);
            let create = OpenOptions::new()
                .create(true)
                .append(true)
                .open(first_file_path.as_str());
            if create.is_ok() {
                data.file = create.unwrap();
                write_last_num(&data.dir_path, data.create_num);
                data.temp_log_num = 0;
                if data.zip_compress {
                    //to zip
                    spawn_to_zip(&current_file_path);
                }
            } else {
                data.create_num -= 1;
            }
        }
        data.file.write(log.as_bytes());
        data.file.flush();
        data.temp_log_num += 1;
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


    let log_file = OpenOptions::new()
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