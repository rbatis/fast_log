use std::time::{Instant};
use fast_log::bencher::QPS;
use fast_log::config::Config;

/// cargo run --release --package example --bin bench_test_file
fn main() {
    //clear data
    std::fs::remove_file("target/test.log");
    fast_log::init(Config::new().file("target/test.log")).unwrap();
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