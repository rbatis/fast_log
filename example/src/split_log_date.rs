use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::error::LogError;
use fast_log::plugin::file_name::FileName;
use fast_log::plugin::file_split::{KeepType, Packer};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

///pack by an date
#[derive(Clone)]
pub struct DateLogPacker {}

impl DateLogPacker {
    pub fn log_name_create_by_time(
        &self,
        first_file_path: &str,
        date: fastdate::DateTime,
    ) -> String {
        let file_name = first_file_path.extract_file_name();
        let mut new_log_name = date.to_string().replace(" ", "T").replace(":", "-");
        new_log_name.push_str(".");
        new_log_name.push_str(self.pack_name());
        new_log_name = first_file_path.trim_end_matches(&file_name).to_string() + &new_log_name;
        return new_log_name;
    }
}
impl Packer for DateLogPacker {
    fn pack_name(&self) -> &'static str {
        "log"
    }

    fn do_pack(&self, mut log_file: File, log_file_path: &str) -> Result<bool, LogError> {
        //do nothing,and not remove file
        let now = fastdate::DateTime::now()
            .set_hour(0)
            .set_min(0)
            .set_sec(0)
            .set_nano(0);
        let name = self.log_name_create_by_time(log_file_path, now);
        let mut f = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .open(&name);
        if let Ok(mut f) = f {
            //append to file
            let mut data = vec![];
            log_file.read_to_end(&mut data)?;
            f.write_all(&data)?;
            std::fs::remove_file(log_file_path)?;
        } else {
            //create file
            f = OpenOptions::new().write(true).create(true).open(name);
            if let Ok(mut f) = f {
                let mut data = vec![];
                log_file.read_to_end(&mut data)?;
                f.write_all(&data)?;
                std::fs::remove_file(log_file_path)?;
            }
        }
        return Ok(false);
    }
}

fn main() {
    //file_path also can use '"target/logs/test.log"'
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        LogSize::MB(1),
        KeepType::KeepNum(2),
        DateLogPacker {},
    ))
    .unwrap();
    for _ in 0..40000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/");
    sleep(Duration::from_secs(3));
}
