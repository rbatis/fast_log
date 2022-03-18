use fast_log::filter::NoFilter;
use fast_log::appender::{FastLogFormatRecord, LogAppender, FastLogRecord};
use std::time::{Instant, Duration};
use fast_log::bencher::QPS;
use fast_log::config::Config;
use fast_log::sleep;

struct BenchRecvLog {}

impl LogAppender for BenchRecvLog {
    fn do_log(&self, record: &FastLogRecord) {
        //do nothing
    }
}

/// cargo run --release --package example --bin bench_test
fn main() {
    fast_log::init(Config::new().custom(BenchRecvLog {}));
    let total = 1000000;
    let now = Instant::now();
    for index in 0..total {
        log::info!("Commencing yak shaving{}", index);
    }
    now.time(total);
    now.qps(total);
    sleep(Duration::from_secs(1));
}