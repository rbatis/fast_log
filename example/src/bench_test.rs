use fast_log::appender::{FastLogRecord, LogAppender};
use fast_log::bencher::TPS;
use fast_log::config::Config;
use std::time::Instant;

/// cargo run --release --package example --bin bench_test
fn main() {
    struct BenchRecvLog {}
    impl LogAppender for BenchRecvLog {
        fn do_logs(&self, _records: &[FastLogRecord]) {
            //nothing
        }
    }
    fast_log::init(Config::new().custom(BenchRecvLog {}).chan_len(Some(100000))).unwrap();
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
