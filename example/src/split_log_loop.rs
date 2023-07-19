use fast_log::config::Config;
use fast_log::consts::LogSize;

///Single logs are stored in rolling mode by capacity
fn main() {
    //or: let file_name = "sloop.log"
    let file_name = "target/logs/sloop.log";
    fast_log::init(
        Config::new()
            .chan_len(Some(100000))
            .console()
            .file_loop(file_name, LogSize::KB(1)),
    )
    .unwrap();
    for _ in 0..80000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", file_name)
}
