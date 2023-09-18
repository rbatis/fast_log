use dyn_clone::DynClone;
use fastdate::DateTime;
use std::{fs::DirEntry, path::Path, time::Duration};

///rolling keep type
#[derive(Copy, Clone, Debug)]
pub enum RollingType {
    /// keep All of log packs
    All,
    /// keep by Time Duration,
    /// for example:
    /// // keep one day log pack
    /// (Duration::from_secs(24 * 3600))
    KeepTime(Duration),
    /// keep log pack num(.log,.zip.lz4...more)
    KeepNum(i64),
}

pub trait Roller: Send + DynClone {
    // fn read_paths(&self, dir: &str, temp_name: &str) -> Vec<DirEntry>;
    fn do_rolling(&self, temp_name: &str, dir: &str) -> i64;
    // fn file_name_parse_time(name: &str, temp_name: &str) -> Option<DateTime>;
}

impl RollingType {
    fn read_paths(&self, dir: &str, temp_name: &str) -> Vec<DirEntry> {
        let base_name = get_base_name(&Path::new(temp_name));
        let paths = std::fs::read_dir(dir);
        if let Ok(paths) = paths {
            //let mut temp_file = None;
            let mut paths_vec = vec![];
            for path in paths {
                match path {
                    Ok(path) => {
                        if let Some(v) = path.file_name().to_str() {
                            if v == temp_name {
                                //temp_file = Some(path);
                                continue;
                            }
                            if !v.starts_with(&base_name) {
                                continue;
                            }
                        }
                        paths_vec.push(path);
                    }
                    _ => {}
                }
            }
            paths_vec.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            // if let Some(v) = temp_file {
            //     paths_vec.push(v);
            // }
            return paths_vec;
        }
        return vec![];
    }

    /// parse `temp2023-07-20T10-13-17.452247.log`
    pub fn file_name_parse_time(name: &str, temp_name: &str) -> Option<DateTime> {
        let base_name = get_base_name(&Path::new(temp_name));
        if name.starts_with(&base_name) {
            let mut time_str = name.trim_start_matches(&base_name).to_string();
            if let Some(v) = time_str.rfind(".") {
                time_str = time_str[0..v].to_string();
            }
            let time = DateTime::parse("YYYY-MM-DDThh:mm:ss.000000", &time_str);
            if let Ok(time) = time {
                return Some(time);
            }
        }
        return None;
    }
}

impl Roller for RollingType {
    fn do_rolling(&self, temp_name: &str, dir: &str) -> i64 {
        let mut removed = 0;
        match self {
            RollingType::KeepNum(n) => {
                let paths_vec = self.read_paths(dir, temp_name);
                for index in 0..paths_vec.len() {
                    if index >= (*n) as usize {
                        let item = &paths_vec[index];
                        std::fs::remove_file(item.path());
                        removed += 1;
                    }
                }
            }
            RollingType::KeepTime(duration) => {
                let paths_vec = self.read_paths(dir, temp_name);
                let now = DateTime::now();
                for index in 0..paths_vec.len() {
                    let item = &paths_vec[index];
                    let file_name = item.file_name();
                    let name = file_name.to_str().unwrap_or("").to_string();
                    if let Some(time) = Self::file_name_parse_time(&name, temp_name) {
                        if now.clone().sub(duration.clone()) > time {
                            std::fs::remove_file(item.path());
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

fn get_base_name(path: &Path) -> String {
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    let p = file_name.rfind(".");
    match p {
        None => file_name,
        Some(i) => file_name[0..i].to_string(),
    }
}
