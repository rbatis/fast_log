use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::{LogPacker};
use fast_log::config::Config;

fn main(){
    fast_log::init(Config::new()
        .console()
        .file_split("target/logs/",
                    LogSize::MB(1),
                    RollingType::All,
                    LogPacker{})).unwrap();
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}","target/logs/")
}