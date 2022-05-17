use log::LevelFilter;
use fast_log::config::Config;

fn main(){
    fast_log::init(Config::new()
        .level(LevelFilter::Debug)
        .console()
        .file("target/requests.log")).unwrap();
    log::debug!("Commencing yak shaving{}", 0);
    log::logger().flush();
}