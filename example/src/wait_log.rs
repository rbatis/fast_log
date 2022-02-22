use fast_log::config::Config;

fn main(){
    let wait=fast_log::init(Config::new()
        .console()
        .file("requests.log")).unwrap();
    log::debug!("Commencing yak shaving{}", 0);
    wait.wait();
}