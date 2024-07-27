use fast_log::config::Config;
use fast_log::plugin::file_split::{RollingType, KeepType, DateType, Rolling};
use std::thread::sleep;
use std::time::Duration;
use fast_log::plugin::packer::LogPacker;

fn main() {
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        Rolling::new(RollingType::ByDate(DateType::Day)),
        KeepType::KeepNum(2),
        LogPacker {},
    ))
        .unwrap();
    for _ in 0..60 {
        sleep(Duration::from_secs(1));
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}
