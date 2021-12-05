use std::thread::sleep;
use std::time::Duration;

fn main(){
    fast_log::init_log("requests.log", log::Level::Debug, None, true).unwrap();
    log::debug!("Commencing yak shaving{}", 0);
    sleep(Duration::from_secs(1));
}