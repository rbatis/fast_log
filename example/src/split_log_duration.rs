use fast_log::config::Config;
use fast_log::plugin::file_split::{PackType, KeepType, DurationType, HowPack};
use std::thread::sleep;
use std::time::Duration;
use fastdate::DateTime;
use fast_log::plugin::packer::LogPacker;


fn main() {
    //file_path also can use '"target/logs/test.log"'
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        HowPack::new(PackType::ByDuration((DateTime::now(), Duration::from_secs(5)))),
        KeepType::KeepNum(5),
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
