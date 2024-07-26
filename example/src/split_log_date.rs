use fast_log::config::Config;
use fast_log::error::LogError;
use fast_log::plugin::file_name::FileName;
use fast_log::plugin::file_split::{HowPackType, KeepType, Packer};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;
use fastdate::DateTime;

///pack by an date
#[derive(Clone)]
pub struct DateLogPacker {}

impl Packer for DateLogPacker {
    fn pack_name(&self) -> &'static str {
        "log"
    }

    fn do_pack(&self, mut log_file: File, log_file_path: &str) -> Result<bool, LogError> {
        impl DateLogPacker {
            pub fn new_log_name(&self, first_file_path: &str, date: fastdate::DateTime) -> String {
                let file_name = first_file_path.extract_file_name();
                let mut new_log_name = date.to_string().replace(" ", "T").replace(":", "-");
                new_log_name.push_str(".");
                new_log_name.push_str(self.pack_name());
                new_log_name =
                    first_file_path.trim_end_matches(&file_name).to_string() + &new_log_name;
                return new_log_name;
            }
        }
        //do nothing,and not remove file
        let now = DateTime::now();
        let name = self.new_log_name(log_file_path, now);
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
        KeepType::KeepNum(2),
        DateLogPacker {},
        HowPackType::ByDate(DateTime::now()),
    ))
        .unwrap();
    for _ in 0..40000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/");
    sleep(Duration::from_secs(3));
}
