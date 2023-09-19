use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::packer::LogPacker;
use fast_log::plugin::roller::DailyRollingType;

fn main() {
    //file_path also can use '"target/logs/test.log"'
    fast_log::init(Config::new().chan_len(Some(100000)).file_daily(
        "target/logs/log.txt",
        LogSize::MB(1),
        DailyRollingType::All,
        LogPacker {},
    ))
    .unwrap();
    for _ in 0..400000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}
