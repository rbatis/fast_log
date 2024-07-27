use fast_log::config::Config;

fn main() {
    fast_log::init(Config::new().console().chan_len(Some(100000))).unwrap();
    log::info!("Commencing yak shaving{}", 0);
    log::error!("Commencing yak shaving{}", 0);
    log::logger().flush();
}
