use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use std::{sync::atomic::AtomicUsize, time::Duration};

use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_name::FileName;
use fast_log::plugin::file_rotate::Rotate;
use fast_log::plugin::file_split::Keep;
use fast_log::plugin::packer::LogPacker;
use fast_log::{appender::FastLogRecord, plugin::file_split::Packer};
use fastdate::{DateTime, DurationFrom};

/// daily rolling keep type
#[derive(Copy, Clone, Debug)]
pub enum DailyKeepType {
    /// keep All of log packs
    All,
    /// keep log pack days, 0 for only today(.log,.zip.lz4...more)
    KeepDays(i64),
}

pub struct DailyKeeper {
    keep_type: DailyKeepType,
    base_name: String,
    date: RwLock<DateTime>,
    rotate_date: RwLock<DateTime>,
    index: AtomicUsize,
}

impl DailyKeeper {
    pub fn new(keep_type: DailyKeepType, base_name: &str) -> Self {
        let today = DateTime::now()
            .set_hour(0)
            .set_min(0)
            .set_sec(0)
            .set_nano(0);

        Self {
            keep_type,
            base_name: base_name.to_string(),
            date: RwLock::new(today.clone()),
            rotate_date: RwLock::new(today.clone().add(Duration::from_day(1))),
            index: AtomicUsize::new(0),
        }
    }

    /// parse `temp_20230720_0.log`
    pub fn file_name_parse_time(name: &str, base_name: &str) -> Option<DateTime> {
        let base_name = Self::get_base_name(base_name);
        if name.starts_with(&base_name) {
            let mut time_str = name.trim_start_matches(&base_name).to_string();
            if let Some(v) = time_str.rfind("_") {
                if v > 1 {
                    time_str = time_str[1..v].to_string();
                }
            }
            let time = DateTime::parse("YYYYMMDD", &time_str);
            if let Ok(time) = time {
                return Some(time);
            }
        }
        return None;
    }

    fn calc_filename(name: &str, index: usize, date: &DateTime) -> String {
        let (name, ext) = name.split_at(name.rfind(".").unwrap());
        format!(
            "{name}_{}_{index}{ext}",
            date.to_string()[0..10].replace("-", "")
        )
    }

    fn get_base_name(path: &str) -> String {
        let file_name = path.extract_file_name();
        let p = file_name.rfind(".");
        match p {
            None => file_name,
            Some(i) => file_name[0..i].to_string(),
        }
    }
}

impl Keep for DailyKeeper {
    fn do_keep(&self, dir: &str, base_name: &str) -> i64 {
        let mut removed = 0;
        match self.keep_type {
            DailyKeepType::KeepDays(n) => {
                let paths_vec = self.read_paths(dir, base_name);
                let now = DateTime::now()
                    .set_hour(0)
                    .set_min(0)
                    .set_sec(0)
                    .set_nano(0);
                for index in 0..paths_vec.len() {
                    let item = &paths_vec[index];
                    let file_name = item.file_name();
                    let name = file_name.to_str().unwrap_or("").to_string();
                    if let Some(time) = Self::file_name_parse_time(&name, base_name) {
                        if now.clone().sub(Duration::from_day(n as u64)) > time {
                            let _ = std::fs::remove_file(item.path());
                            removed += 1;
                        }
                    }
                }
            }
            _ => {}
        }
        removed
    }
}

impl Rotate for DailyKeeper {
    fn base_name(&self) -> &str {
        self.base_name.as_str()
    }

    fn init(&self, dir_path: &str, packer: &Box<dyn Packer>) -> String {
        let path = Path::new(&dir_path);
        let date = self.date.read().unwrap();
        let mut pack_not_found = false;
        for i in 0..usize::MAX {
            let name = path.join(Self::calc_filename(self.base_name(), i + 1, &date));
            let packed_name = name.with_extension(packer.pack_name());
            if !pack_not_found {
                match std::fs::metadata(packed_name) {
                    Ok(_) => continue,
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            self.index.store(i, Ordering::Relaxed);
                            pack_not_found = true;
                        }
                        _ => panic!("{:?}", e),
                    },
                }
            }
            if pack_not_found {
                match std::fs::metadata(name) {
                    Ok(_) => continue,
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            self.index.store(i, Ordering::Relaxed);
                            break;
                        }
                        _ => panic!("{:?}", e),
                    },
                }
            }
        }
        self.current()
    }

    fn current(&self) -> String {
        Self::calc_filename(
            self.base_name(),
            self.index.load(Ordering::Relaxed),
            &self.date.read().unwrap(),
        )
    }

    fn next(&self, record: &FastLogRecord) -> String {
        if self.should_rotate(record) {
            let record_date = fastdate::DateTime::from(record.now)
                .add_sub_sec(fastdate::offset_sec() as i64)
                .set_hour(0)
                .set_min(0)
                .set_sec(0)
                .set_nano(0);

            self.index.store(0, Ordering::Relaxed);
            *self.date.write().unwrap() = record_date.clone();
            *self.rotate_date.write().unwrap() = record_date.clone().add(Duration::from_day(1));
            self.current()
        } else {
            self.index.fetch_add(1, Ordering::Relaxed);
            self.current()
        }
    }

    fn should_rotate(&self, record: &FastLogRecord) -> bool {
        let record_date = fastdate::DateTime::from(record.now)
            .add_sub_sec(fastdate::offset_sec() as i64)
            .set_hour(0)
            .set_min(0)
            .set_sec(0)
            .set_nano(0);
        record_date >= *self.rotate_date.read().unwrap()
    }
}

fn main() {
    // file_path also can use '"target/logs/test.log"'
    fast_log::init(Config::new().chan_len(Some(100000)).file_rotate(
        "target/logs/",
        LogSize::MB(1),
        DailyKeeper::new(DailyKeepType::All, "log.txt"),
        LogPacker {},
    ))
    .unwrap();
    for _ in 0..400000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use fast_log::plugin::packer::LogPacker;
    use std::time::Duration;

    #[test]
    fn test_log_dailykeeper() {
        let keeper = DailyKeeper::new(DailyKeepType::All, "log.txt");
        let log_packer = LogPacker {};
        let today = DateTime::now()
            .set_hour(0)
            .set_min(0)
            .set_sec(0)
            .set_nano(0);
        let date_str = |date: &DateTime| date.to_string()[0..10].replace("-", "");
        let today_date_str = date_str(&today);
        let today_record = FastLogRecord {
            command: fast_log::appender::Command::CommandRecord,
            level: log::Level::Info,
            target: String::default(),
            args: String::default(),
            module_path: String::default(),
            file: String::default(),
            line: None,
            now: today.clone().into(),
            formated: String::default(),
        };
        let tomorrow = today.clone().add(Duration::from_day(1));
        let tomorrow_record = FastLogRecord {
            command: fast_log::appender::Command::CommandRecord,
            level: log::Level::Info,
            target: String::default(),
            args: String::default(),
            module_path: String::default(),
            file: String::default(),
            line: None,
            now: tomorrow.clone().into(),
            formated: String::default(),
        };
        let tomorrow_date_str = date_str(&tomorrow);

        assert_eq!("log", DailyKeeper::get_base_name("log.txt"));
        assert_eq!(
            format!("log_{today_date_str}_0.txt"),
            DailyKeeper::calc_filename("log.txt", 0, &today)
        );
        assert_eq!(
            Some(today),
            DailyKeeper::file_name_parse_time(&format!("log_{today_date_str}_0.txt"), "log.txt")
        );
        assert_eq!(
            Some(tomorrow),
            DailyKeeper::file_name_parse_time(&format!("log_{tomorrow_date_str}_0.txt"), "log.txt")
        );
        assert_eq!(
            None,
            DailyKeeper::file_name_parse_time(&format!("log__0.txt"), "log.txt")
        );
        assert_eq!(
            None,
            DailyKeeper::file_name_parse_time(&format!("log_0.txt"), "log.txt")
        );
        assert_eq!(
            None,
            DailyKeeper::file_name_parse_time(&format!("log0.txt"), "log.txt")
        );
        assert_eq!(
            None,
            DailyKeeper::file_name_parse_time(&format!("log.txt"), "log.txt")
        );
        assert_eq!("log.txt", keeper.base_name());
        assert_eq!(format!("log_{today_date_str}_0.txt"), keeper.current());
        assert_eq!(
            format!("log_{today_date_str}_0.txt"),
            keeper.init("logs/", &(Box::new(log_packer) as Box<dyn Packer>))
        );
        assert_eq!(
            format!("log_{today_date_str}_1.txt"),
            keeper.next(&today_record)
        );
        assert_eq!(false, keeper.should_rotate(&today_record));
        assert_eq!(
            format!("log_{today_date_str}_2.txt"),
            keeper.next(&today_record)
        );
        assert_eq!(true, keeper.should_rotate(&tomorrow_record));
        assert_eq!(
            format!("log_{tomorrow_date_str}_0.txt"),
            keeper.next(&tomorrow_record)
        );
        assert_eq!(
            format!("log_{tomorrow_date_str}_1.txt"),
            keeper.next(&tomorrow_record)
        );
    }
}
