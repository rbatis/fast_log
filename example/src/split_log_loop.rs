use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::{LogPacker};
use std::thread::sleep;
use std::time::Duration;
use fast_log::appender::FastLogFormatRecord;
use fast_log::filter::NoFilter;
use fast_log::plugin::file_loop::FileLoopAppender;

///Single logs are stored in rolling mode by capacity
fn main(){
    fast_log::init_custom_log(vec![Box::new(FileLoopAppender::new("target/logs/sloop.log",LogSize::KB(1)))],
                              log::Level::Info,
                              Box::new(NoFilter {}),
                              Box::new(FastLogFormatRecord::new()),
    );
    for _ in 0..80000 {
        log::info!("Commencing yak shaving");
    }
    sleep(Duration::from_secs(3));
    println!("you can see log files in path: {}","target/logs/")
}