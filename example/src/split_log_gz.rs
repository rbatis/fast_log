use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::{PackType, KeepType};
use fast_log::plugin::packer::GZipPacker;

fn main() {
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        PackType::BySize(LogSize::KB(50)),
        KeepType::KeepNum(5),
        GZipPacker {},
    ))
    .unwrap();
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/");
}
