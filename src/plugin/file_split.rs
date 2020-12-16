use std::cell::RefCell;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use chrono::Local;
use zip::write::FileOptions;

use crate::appender::{FastLogRecord, LogAppender};
use crate::consts::LogSize;

const SPLITE_CONFIG_NAME: &str = ".fast_log_split_appender";

/// split log file allow zip compress log
pub struct FileSplitAppender {
    cell: RefCell<FileSplitAppenderData>
}

/// split log file allow zip compress log
pub struct FileSplitAppenderData {
    max_split_bytes: usize,
    temp_bytes: usize,
    create_num: u64,
    dir_path: String,
    file: File,
    zip_compress: bool,
}

impl FileSplitAppender {
    ///split_log_bytes: log file data bytes(MB) splite
    ///dir_path the dir
    pub fn new(dir_path: &str, max_temp_size: LogSize, allow_zip_compress: bool) -> FileSplitAppender {
        if !dir_path.is_empty() && dir_path.ends_with(".log") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() && !dir_path.ends_with("/") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() {
            DirBuilder::new().create(dir_path);
        }
        let last = open_last_num(dir_path).unwrap();
        let first_file_path = format!("{}{}.log", dir_path.to_string(), last);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(first_file_path.as_str());
        if file.is_err() {
            panic!("[fast_log] open and create file fail:{}", file.err().unwrap());
        }
        let file = file.unwrap();
        let mut temp_bytes = 0;
        match file.metadata() {
            Ok(m) => {
                temp_bytes = m.len() as usize;
            }
            _ => {}
        }
        Self {
            cell: RefCell::new(FileSplitAppenderData {
                max_split_bytes: max_temp_size.get_len(),
                temp_bytes: temp_bytes,
                create_num: last,
                dir_path: dir_path.to_string(),
                file: file,
                zip_compress: allow_zip_compress,
            })
        }
    }
}

impl LogAppender for FileSplitAppender {
    fn do_log(&self, record: &FastLogRecord) {
        let log_data = record.formated.as_str();
        let mut data = self.cell.borrow_mut();
        if data.temp_bytes >= data.max_split_bytes {
            let current_file_path = format!("{}{}.log", data.dir_path.to_string(), data.create_num);
            let next_file_path = format!("{}{}.log", data.dir_path.to_string(), data.create_num + 1);
            let create = OpenOptions::new()
                .create(true)
                .append(true)
                .open(next_file_path.as_str());
            match create {
                Ok(next_file) => {
                    data.temp_bytes = 0;
                    data.create_num += 1;
                    data.file = next_file;
                    write_last_num(&data.dir_path, data.create_num);
                    if data.zip_compress {
                        //to zip
                        spawn_to_zip(&current_file_path);
                    }
                }
                _ => {}
            }
        }
        let write_bytes = data.file.write(log_data.as_bytes());
        data.file.flush();
        match write_bytes {
            Ok(size) => {
                data.temp_bytes += size;
            }
            _ => {}
        }
    }
}

fn open_last_num(dir_path: &str) -> Result<u64, String> {
    let mut config = OpenOptions::new()
        .read(true)
        .open(Path::new(format!("{}{}", dir_path, SPLITE_CONFIG_NAME).as_str()));
    if config.is_err() {
        config = File::create(Path::new(format!("{}{}", dir_path, SPLITE_CONFIG_NAME).as_str()));
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
        .open(Path::new(format!("{}{}", dir_path, SPLITE_CONFIG_NAME).as_str()))
        .unwrap();
    config.write(last.to_string().as_bytes());
    config.flush();
}

fn spawn_to_zip(log_file: &str) {
    let log_file = log_file.to_owned();
    std::thread::spawn(move || {
        do_zip(log_file.as_str());
    });
}

fn do_zip(log_file_path: &str) {
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
            let date = Local::now();
            let date = date.format("%Y_%m_%dT%H_%M_%S").to_string();
            let zip_path = log_file_path.replace(".log", &format!("_{}.zip", date));
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
                        Err(e) => {
                            //nothing
                            println!("[fast_log] try zip fail{:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("[fast_log] create(&{}) fail:{}", zip_path, e);
                }
            }
        }
        Err(e) => {
            println!("[fast_log] give up compress log file. because: {}", e);
        }
    }
}