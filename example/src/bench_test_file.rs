use fast_log::bencher::TPS;
use fast_log::config::Config;
use std::time::Instant;

/// cargo run --release --package example --bin bench_test_file
fn main() {
    //clear data
    let _ = std::fs::remove_file("target/test.log");
    fast_log::init(Config::new().file("target/test.log")).unwrap();
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
