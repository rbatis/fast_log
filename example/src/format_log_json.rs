use fast_log::config::Config;
use fast_log::FastLogFormatJson;
use log::LevelFilter;

fn main() {
    fast_log::init(Config::new().format(FastLogFormatJson::new()).console()).unwrap();
    log::info!("Commencing yak shaving{}", 0);
    log::logger().flush();
}
