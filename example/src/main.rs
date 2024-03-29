use fast_log::config::Config;

fn main() {
    fast_log::init(Config::new().console().chan_len(Some(100000))).unwrap();
    for _ in 0..100 {
        log::info!("Commencing yak shaving{}", 0);
    }
    fast_log::print("Commencing print\n".into()).expect("fast log not init");
    log::error!("Commencing yak shaving{}", 0);
    log::logger().flush();
}
