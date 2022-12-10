use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::LogPacker;

fn main() {
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        LogSize::MB(1),
        RollingType::All,
        LogPacker {},
    ))
    .unwrap();
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }

    log::logger().flush();

    // /// Even if the capacity is not reached, a log is forced to save
    // let wg = fast_log::flush().unwrap();
    // /// wait save end,or you can use
    // /// let wait = fast_log::init_split_log(...);
    // /// wait.wait();
    // ///
    // wg.wait();
    println!("you can see log files in path: {}", "target/logs/")
}
