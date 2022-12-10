use fast_log::config::Config;
use fast_log::consts::LogSize;

///Single logs are stored in rolling mode by capacity
fn main() {
    fast_log::init(
        Config::new()
            .console()
            .file_loop("target/logs/sloop.log", LogSize::KB(1))
            .chan_len(Some(100000)),
    )
    .unwrap();
    for _ in 0..80000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}
