use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::{ZipPacker, LogPacker};
use std::thread::sleep;
use std::time::Duration;

fn main(){
    fast_log::init_split_log(
        "target/logs/",
        1000,
        LogSize::MB(1),
        RollingType::All,
        log::Level::Info,
        None,
        Box::new(LogPacker{}),
        true,
    );
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }
    sleep(Duration::from_secs(1));
    println!("you can see log files in path: {}","target/logs/")
}