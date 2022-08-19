use fast_log::config::Config;
use fast_log::FastLogFormat;
use log::LevelFilter;

fn main() {
    fast_log::init(
        Config::new()
            .format(FastLogFormat::new().set_display_line_level(LevelFilter::Trace))
            .console(),
    )
    .unwrap();
    log::info!("Commencing yak shaving{}", 0);
    log::logger().flush();
}
