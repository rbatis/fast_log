use fast_log::config::Config;
use log::Level;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let log = fast_log::init(Config::new().console()).unwrap();
    log::info!("Commencing yak shaving{}", 0);
    sleep(Duration::from_secs(1));
}
