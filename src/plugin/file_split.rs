use std::cell::RefCell;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

use chrono::Local;
use crossbeam_channel::{Receiver, Sender};
use zip::write::FileOptions;

use crate::appender::{FastLogRecord, LogAppender};
use crate::consts::LogSize;

/// split log file allow zip compress log
pub struct FileSplitAppender {
    cell: RefCell<FileSplitAppenderData>
}


///log data pack
pub struct LogPack {
    pub info: String,
    pub data: Vec<u8>,
    pub log_file_name: String,
}

/// split log file allow zip compress log
pub struct FileSplitAppenderData {
    max_split_bytes: usize,
    dir_path: String,
    file: File,
    zip_compress: bool,
    sender: Sender<LogPack>,
    //cache data
    temp_bytes: usize,
    temp_data: Option<Vec<u8>>,
}

impl FileSplitAppenderData {
    pub fn send_pack(&mut self) {
        let log_name = format!("{}{}{}.log", self.dir_path, "temp", format!("{:29}", Local::now().format("%Y_%m_%dT%H_%M_%S%.f")).replace(" ", "_"));
        if self.zip_compress {
            //to zip
            match self.temp_data.take() {
                Some(temp) => {
                    self.sender.send(LogPack {
                        info: "zip".to_string(),
                        data: temp,
                        log_file_name: log_name,
                    });
                }
                _ => {}
            }
        } else {
            //send data
            let log_data = self.temp_data.take().unwrap();
            self.sender.send(LogPack {
                info: "log".to_string(),
                data: log_data,
                log_file_name: log_name,
            });
        }
        self.truncate();
    }

    pub fn truncate(&mut self) {
        //reset data
        self.file.set_len(0);
        self.file.seek(SeekFrom::Start(0));
        self.temp_bytes = 0;
        self.temp_data = Some(vec![]);
    }
}


impl FileSplitAppender {
    ///split_log_bytes:  log file data bytes(MB) splite
    ///dir_path:         the log dir
    ///log_pack_cap:     zip or log Waiting cap
    pub fn new(dir_path: &str, max_temp_size: LogSize, allow_zip_compress: bool, log_pack_cap: usize) -> FileSplitAppender {
        if !dir_path.is_empty() && dir_path.ends_with(".log") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() && !dir_path.ends_with("/") {
            panic!("FileCompactionAppender only support new from path,for example: 'logs/xx/'");
        }
        if !dir_path.is_empty() {
            DirBuilder::new().create(dir_path);
        }
        let first_file_path = format!("{}{}.log", dir_path, "temp");
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(first_file_path.as_str());
        if file.is_err() {
            panic!("[fast_log] open and create file fail:{}", file.err().unwrap());
        }
        let mut file = file.unwrap();
        let mut temp_bytes = 0;
        match file.metadata() {
            Ok(m) => {
                temp_bytes = m.len() as usize;
            }
            _ => {}
        }
        let mut temp_data = vec![];
        file.read_to_end(&mut temp_data);
        file.seek(SeekFrom::Start(temp_bytes as u64));
        let (s, r) = crossbeam_channel::bounded(log_pack_cap);
        spawn_saver_thread(r);
        Self {
            cell: RefCell::new(FileSplitAppenderData {
                max_split_bytes: max_temp_size.get_len(),
                temp_bytes: temp_bytes,
                temp_data: Some(temp_data),
                dir_path: dir_path.to_string(),
                file: file,
                zip_compress: allow_zip_compress,
                sender: s,
            })
        }
    }
}

impl LogAppender for FileSplitAppender {
    fn do_log(&self, record: &FastLogRecord) {
        let log_data = record.formated.as_str();
        let mut data = self.cell.borrow_mut();
        if data.temp_bytes >= data.max_split_bytes {
            data.send_pack();
        }
        let write_bytes = data.file.write(log_data.as_bytes());
        data.file.flush();
        match write_bytes {
            Ok(size) => {
                let bytes = log_data.as_bytes();
                data.temp_data.as_mut().unwrap().write_all(bytes);
                data.temp_bytes += size;
            }
            _ => {}
        }
    }
}

///spawn an saver thread to save log file or zip file
fn spawn_saver_thread(r: Receiver<LogPack>) {
    std::thread::spawn(move || {
        loop {
            match r.recv() {
                Ok(pack) => {
                    match pack.info.as_str() {
                        "zip" => {
                            do_zip(pack);
                        }
                        "log" => {
                            do_log(pack);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });
}

///write an ZipPack to log file
pub fn do_log(pack: LogPack) {
    let f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(pack.log_file_name);
    match f {
        Ok(mut f) => {
            f.write_all(&pack.data);
            f.flush();
        }
        _ => {}
    }
}

/// write an ZipPack to zip file
pub fn do_zip(pack: LogPack) {
    let log_file_path = pack.log_file_name.as_str();
    if log_file_path.is_empty() || pack.data.is_empty() {
        return;
    }


    let mut log_name = log_file_path.replace("\\", "/").to_string();
    let bn = log_name.as_str();
    match log_file_path.rfind("/") {
        Some(v) => {
            log_name = log_name[(v + 1)..log_name.len()].to_string();
        }
        _ => {}
    }
    let af = log_name.as_str();
    //make zip
    let zip_path = log_file_path.replace(".log", ".zip");
    let zip_file = std::fs::File::create(&zip_path);
    if zip_file.is_err() {
        println!("[fast_log] create(&{}) fail:{}", zip_path, zip_file.err().unwrap());
        return;
    }
    let zip_file = zip_file.unwrap();
    //write zip bytes data
    let mut zip = zip::ZipWriter::new(zip_file);
    zip.start_file(log_name, FileOptions::default());
    zip.write_all(pack.data.as_slice());
    zip.flush();
    let finish = zip.finish();
    if finish.is_err() {
        println!("[fast_log] try zip fail{:?}", finish.err());
        return;
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use zip::write::FileOptions;

    #[test]
    fn test_zip() {
        let zip_file = std::fs::File::create("F:/rust_project/fast_log/target/logs/0.zip");
        match zip_file {
            Ok(zip_file) => {
                let mut zip = zip::ZipWriter::new(zip_file);
                zip.start_file("0.log", FileOptions::default());
                zip.write("sadfsadfsadf".as_bytes());
                let finish = zip.finish();
                match finish {
                    Ok(f) => {
                        //std::fs::remove_file("F:/rust_project/fast_log/target/logs/0.log");
                    }
                    Err(e) => {
                        //nothing
                        panic!(e)
                    }
                }
            }
            Err(e) => {
                panic!(e)
            }
        }
    }
}