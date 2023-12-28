use fast_log::bencher::TPS;
use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::KeepType;
use fast_log::plugin::packer::LogPacker;
use std::time::Instant;

/// cargo run --release --package example --bin bench_test_file_split
fn main() {
    //clear data
    let _ = std::fs::remove_dir("target/logs/");
    fast_log::init(
        Config::new()
            .file_split("target/logs/", LogSize::MB(1), KeepType::All, LogPacker {})
            .chan_len(Some(100000)),
    )
    .unwrap();
    log::info!("Commencing yak shaving{}", 0);
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
