use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::cleaner::{RollingAll, RollingNum};
use fast_log::plugin::packer::LogPacker;

fn main() {
    //file_path also can use '"target/logs/test.log"'
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        LogSize::MB(1),
        RollingNum(2),
        LogPacker {},
    ))
    .unwrap();
    for _ in 0..40000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}
