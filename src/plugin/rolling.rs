use crate::plugin::file_split::Rolling;
use fastdate::DateTime;
use std::time::Duration;

/// keeps all,do not rolling
pub struct RollingAll {}
impl Rolling for RollingAll {
    fn do_rolling(&self, dir: &str, temp_name: &str) -> i64 {
        0
    }
}

/// rolling from file num
pub struct RollingNum {
    pub num: i64,
}

impl Rolling for RollingNum {
    fn do_rolling(&self, dir: &str, temp_name: &str) -> i64 {
        let mut removed = 0;
        let paths_vec = self.read_paths(dir, temp_name);
        for index in 0..paths_vec.len() {
            if index >= (self.num) as usize {
                let item = &paths_vec[index];
                std::fs::remove_file(item.path());
                removed += 1;
            }
        }
        removed
    }
}

/// rolling from metadata
pub struct RollingDuration {
    pub duration: Duration,
}

impl Rolling for RollingDuration {
    fn do_rolling(&self, dir: &str, temp_name: &str) -> i64 {
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
                    if now.clone().sub(self.duration.clone()) > time {
                        std::fs::remove_file(item.path());
                        removed += 1;
                    }
                }
            }
        }
        removed
    }
}
