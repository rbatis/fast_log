use crate::plugin::file_split::Packer;
use std::fs::File;
use crate::error::LogError;
use zip::write::FileOptions;
use std::io::{BufReader, Write, BufRead};
use zip::result::ZipResult;

/// the zip compress
pub struct ZipPacker {}

impl Packer for ZipPacker {
    fn pack_name(&self) -> &'static str {
        "zip"
    }

    fn do_pack(&self, log_file: File, log_file_path: &str) -> Result<(), LogError> {
        let mut log_name = log_file_path.replace("\\", "/").to_string();
        match log_file_path.rfind("/") {
            Some(v) => {
                log_name = log_name[(v + 1)..log_name.len()].to_string();
            }
            _ => {}
        }
        let zip_path = log_file_path.replace(".log", ".zip");
        let zip_file = std::fs::File::create(&zip_path);
        if zip_file.is_err() {
            return Err(LogError::from(format!(
                "[fast_log] create(&{}) fail:{}",
                zip_path,
                zip_file.err().unwrap()
            )));
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
        let finish: ZipResult<File> = zip.finish();
        if finish.is_err() {
            //println!("[fast_log] try zip fail{:?}", finish.err());
            return Err(LogError::from(format!("[fast_log] try zip fail{:?}", finish.err())));
        }
        return Ok(());
    }
}