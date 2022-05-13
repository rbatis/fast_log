use fast_log::config::Config;

fn main() {
    fast_log::init(Config::new().console()).unwrap();
    log::info!("Commencing yak shaving{}", 0);
    log::logger().flush();
}