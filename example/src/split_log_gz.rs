use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use std::thread::sleep;
use std::time::Duration;
use fast_log::config::Config;
use fast_log::plugin::packer::GZipPacker;

fn main(){
    fast_log::init(Config::new()
        .console()
        .file_split("target/logs/",
                    LogSize::KB(50),
                    RollingType::KeepNum(5),
                    GZipPacker{})).unwrap();
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }
    sleep(Duration::from_secs(1));
    println!("you can see log files in path: {}","target/logs/")
}