use fast_log::consts::{LogSize, SplitType};
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::ZipPacker;

use fast_log::config::Config;

fn main() {
    fast_log::init(Config::new().console().file_split(
        "target/logs/",
        SplitType::Size(LogSize::KB(50)),
        RollingType::KeepNum(5),
        ZipPacker {},
    ))
    .unwrap();
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}
