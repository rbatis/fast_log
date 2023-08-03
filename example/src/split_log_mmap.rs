use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_mmap::MmapFile;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::LogPacker;

fn main() {
    fast_log::init(
        Config::new()
            .chan_len(Some(100000))
            .console()
            .split::<MmapFile, LogPacker>(
                "target/logs/",
                LogSize::MB(1),
                RollingType::All,
                LogPacker {},
            ),
    )
    .unwrap();
    for _ in 0..40000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}
