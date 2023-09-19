use crate::plugin::file_split::Cleaner;
use fastdate::DateTime;
use std::time::Duration;

/// keeps all,do not rolling
pub struct RollingAll {}
impl Cleaner for RollingAll {
    fn do_clean(&self, dir: &str, temp_name: &str) -> i64 {
        0
    }
}

/// rolling from file num
pub struct RollingNum(pub i64);

impl Cleaner for RollingNum {
    fn do_clean(&self, dir: &str, temp_name: &str) -> i64 {
        let mut removed = 0;
        let paths_vec = self.read_paths(dir, temp_name);
        for index in 0..paths_vec.len() {
            if index >= (self.0) as usize {
                let item = &paths_vec[index];
                std::fs::remove_file(item.path());
                removed += 1;
            }
        }
        removed
    }
}

/// rolling from metadata
pub struct RollingDuration(pub Duration);

impl Cleaner for RollingDuration {
    fn do_clean(&self, dir: &str, temp_name: &str) -> i64 {
        let mut removed = 0;
        let paths_vec = self.read_paths(dir, temp_name);
        let now = DateTime::now();
        for index in 0..paths_vec.len() {
            let item = &paths_vec[index];
            let file_name = item.file_name();
            let name = file_name.to_str().unwrap_or("").to_string();
            if let Ok(m) = item.metadata() {
                if let Ok(c) = m.created() {
                    let time = DateTime::from(c);
                    if now.clone().sub(self.0.clone()) > time {
                        std::fs::remove_file(item.path());
                        removed += 1;
                    }
                }
            }
        }
        removed
    }
}
