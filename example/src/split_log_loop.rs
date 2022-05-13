use fast_log::appender::FastLogFormatRecord;
use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::filter::NoFilter;
use fast_log::plugin::file_loop::FileLoopAppender;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::LogPacker;
use std::thread::sleep;
use std::time::Duration;

///Single logs are stored in rolling mode by capacity
fn main() {
    fast_log::init(
        Config::new()
            .console()
            .file_loop("target/logs/sloop.log", LogSize::KB(1)),
    )
    .unwrap();
    for _ in 0..80000 {
        log::info!("Commencing yak shaving");
    }
    sleep(Duration::from_secs(3));
    println!("you can see log files in path: {}", "target/logs/")
}
