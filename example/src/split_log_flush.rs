use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::{LogPacker};
use std::thread::sleep;
use std::time::Duration;
use fast_log::config::Config;

fn main(){
    fast_log::init(Config::new()
        .console()
        .file_split("target/logs/",
                    LogSize::MB(1),
                    RollingType::All,
                    LogPacker{})).unwrap();
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }
    /// Even if the capacity is not reached, a log is forced to save
    fast_log::flush();
    /// wait save end,or you can use
    /// let wait = fast_log::init_split_log(...);
    /// wait.wait();
    ///
    sleep(Duration::from_secs(3));
    println!("you can see log files in path: {}","target/logs/")
}