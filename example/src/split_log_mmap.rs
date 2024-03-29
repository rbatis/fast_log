use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_mmap::MmapFile;
use fast_log::plugin::file_split::KeepType;
use fast_log::plugin::packer::LogPacker;

fn main() {
    fast_log::init(
        Config::new()
            .chan_len(Some(100000))
            .console()
            .split::<MmapFile, KeepType, LogPacker>(
                "target/logs/temp.log",
                LogSize::MB(1),
                KeepType::All,
                LogPacker {},
            ),
    )
    .unwrap();
    for _ in 0..40000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/");
}
