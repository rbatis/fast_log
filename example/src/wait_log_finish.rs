fn main(){
    let wait=fast_log::init_log("requests.log", 1000, log::Level::Debug, None, true).unwrap();
    log::debug!("Commencing yak shaving{}", 0);
    wait.wait();
}