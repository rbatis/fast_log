use fast_log::appender::FastLogRecord;
use fast_log::bencher::TPS;
use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_name::FileName;
use fast_log::plugin::file_rotate::Rotate;
use fast_log::plugin::file_split::{Keep, Packer};
use fast_log::plugin::packer::LogPacker;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// daily rolling keep type
#[derive(Copy, Clone, Debug)]
pub enum RotateKeepType {
    /// keep All of log packs
    All,
    /// keep log pack days, 0 for only today(.log,.zip.lz4...more)
    KeepNum(i64),
}

pub struct RotateKeeper {
    keep_type: RotateKeepType,
    index: AtomicUsize,
}

impl RotateKeeper {
    pub fn new(keep_type: RotateKeepType) -> Self {
        Self {
            keep_type,
            index: AtomicUsize::new(0),
        }
    }

    /// parse `temp_0.log`
    pub fn file_name_parse_index(name: &str, temp_name: &str) -> Option<usize> {
        let base_name = Self::get_base_name(temp_name);
        if name.starts_with(&base_name) {
            let mut index_str = name.trim_start_matches(&base_name).to_string();
            if let Some(v) = index_str.rfind(".") {
                index_str = index_str[1..v].to_string();
            }
            let index = index_str.parse();
            if let Ok(time) = index {
                return Some(time);
            }
        }
        return None;
    }

    fn calc_filename(name: &str, index: usize) -> String {
        let (name, ext) = name.split_at(name.rfind(".").unwrap());
        format!("{name}_{index}{ext}")
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

impl Keep for RotateKeeper {
    fn do_keep(&self, dir: &str, temp_name: &str) -> i64 {
        let mut removed = 0;
        match self.keep_type {
            RotateKeepType::KeepNum(n) => {
                let mut paths_vec = self.read_paths(dir, temp_name);
                paths_vec.sort_by(|a, b| {
                    Self::file_name_parse_index(b.file_name().to_str().unwrap(), temp_name).cmp(
                        &Self::file_name_parse_index(a.file_name().to_str().unwrap(), temp_name),
                    )
                });
                for index in 0..paths_vec.len() {
                    if index >= n as usize {
                        let item = &paths_vec[index];
                        let _ = std::fs::remove_file(item.path());
                        removed += 1;
                    }
                }
            }
            _ => {}
        }
        removed
    }
}

impl Rotate for RotateKeeper {
    fn base_name(&self) -> &str {
        "temp.log"
    }

    fn init(&self, dir_path: &str, _packer: &Box<dyn Packer>) -> String {
        let max_index = self
            .read_paths(dir_path, self.base_name())
            .iter()
            .map(|it| {
                Self::file_name_parse_index(it.file_name().to_str().unwrap(), self.base_name())
            })
            .filter(|it| it.is_some())
            .map(|it| it.unwrap())
            .max();
        self.index
            .store(max_index.unwrap_or_default(), Ordering::Relaxed);
        self.current()
    }

    fn current(&self) -> String {
        Self::calc_filename(self.base_name(), self.index.load(Ordering::Relaxed))
    }

    fn next(&self, _record: &FastLogRecord) -> String {
        self.index.fetch_add(1, Ordering::Relaxed);
        self.current()
    }

    fn should_rotate(&self, _record: &FastLogRecord) -> bool {
        false
    }
}

/// cargo run --release --package example --bin bench_test_file_rotate
fn main() {
    //clear data
    let _ = std::fs::remove_dir("target/logs/");
    fast_log::init(
        Config::new()
            .file_rotate(
                "target/logs/log.txt",
                LogSize::MB(1),
                RotateKeeper::new(RotateKeepType::KeepNum(100)),
                LogPacker {},
            )
            .chan_len(Some(100000)),
    )
    .unwrap();
    log::info!("Commencing yak shaving{}", 0);
    let total = 1000000;
    let now = Instant::now();
    for index in 0..total {
        log::info!("Commencing yak shaving{}", index);
    }
    //wait log finish write all
    log::logger().flush();
    now.time(total);
    now.tps(total);
}
