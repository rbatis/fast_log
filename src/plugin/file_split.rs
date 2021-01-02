use std::cell::RefCell;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Write, Seek, SeekFrom, BufReader, BufRead};

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
    pub new_log_name: String,
}

/// split log file allow zip compress log
/// Memory space swop running time , reduces the number of repeated queries for IO
pub struct FileSplitAppenderData {
    max_split_bytes: usize,
    dir_path: String,
    file: File,
    zip_compress: bool,
    sender: Sender<LogPack>,
    //cache data
    temp_bytes: usize,
}

impl FileSplitAppenderData {
    pub fn send_pack(&mut self) {
        let first_file_path = format!("{}{}.log", self.dir_path, "temp");
        let new_log_name = format!("{}{}{}.log", self.dir_path, "temp", format!("{:29}", Local::now().format("%Y_%m_%dT%H_%M_%S%.f")).replace(" ", "_"));
        std::fs::copy(&first_file_path, &new_log_name);
        if self.zip_compress {
            //to zip
            self.sender.send(LogPack {
                info: "zip".to_string(),
                new_log_name: new_log_name,
            });
        } else {
            //send data
            self.sender.send(LogPack {
                info: "log".to_string(),
                new_log_name: new_log_name,
            });
        }
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
        file.seek(SeekFrom::Start(temp_bytes as u64));
        let (s, r) = crossbeam_channel::bounded(log_pack_cap);
        spawn_saver_thread(r);
        Self {
            cell: RefCell::new(FileSplitAppenderData {
                max_split_bytes: max_temp_size.get_len(),
                temp_bytes: temp_bytes,
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
                            //nothing to do
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });
}


/// write an ZipPack to zip file
pub fn do_zip(pack: LogPack) {
    let log_file_path = pack.new_log_name.as_str();
    if log_file_path.is_empty() {
        return;
    }
    let log_file = OpenOptions::new()
        .read(true)
        .open(log_file_path);
    if log_file.is_err() {
        return;
    }
    let log_file = log_file.unwrap();
    let mut log_name = log_file_path.replace("\\", "/").to_string();
    match log_file_path.rfind("/") {
        Some(v) => {
            log_name = log_name[(v + 1)..log_name.len()].to_string();
        }
        _ => {}
    }
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
    //buf reader
    let mut r = BufReader::new(log_file);
    let mut buf = String::new();
    while let Ok(l) = r.read_line(&mut buf) {
        if l == 0 {
            break;
        }
        zip.write(buf.as_bytes());
        buf.clear();
    }
    zip.flush();
    let finish = zip.finish();
    if finish.is_err() {
        println!("[fast_log] try zip fail{:?}", finish.err());
        return;
    }
    std::fs::remove_file(log_file_path);
}

#[cfg(test)]
mod test {
    use std::io::{Write, BufReader, BufRead};

    use zip::write::FileOptions;
    use std::fs::{OpenOptions};

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

    #[test]
    fn test_buf() {
        let log_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("target/logs/test.log").unwrap();
        //buf reader
        let mut r = BufReader::new(log_file);
        let mut buf = String::new();
        while let Ok(l) = r.read_line(&mut buf) {
            if l == 0 {
                break;
            }
            print!("{}", buf);
            buf.clear();
        }
    }
}