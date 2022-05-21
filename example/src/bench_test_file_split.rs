use std::time::{Instant};
use fast_log::bencher::QPS;
use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::LogPacker;

/// cargo run --release --package example --bin bench_test_file_split
fn main() {
    //clear data
    std::fs::remove_dir("target/logs/");
    fast_log::init(Config::new().file_split("target/logs/",
                                             LogSize::MB(1),
                                             RollingType::All,
                                             LogPacker{})).unwrap();
    log::info!("Commencing yak shaving{}", 0);
    let total = 1000000;
    let now = Instant::now();
    for index in 0..total {
        log::info!("Commencing yak shaving{}", index);
    }
    //wait log finish write all
    log::logger().flush();
    now.time(total);
    now.qps(total);
}