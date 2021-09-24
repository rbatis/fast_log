use std::time::Duration;
use fast_log::appender::{FastLogFormatRecord, LogAppender, FastLogRecord};
use fast_log::filter::NoFilter;
use log::Level;
use std::thread::sleep;

struct CustomLog {}

impl LogAppender for CustomLog {
    fn do_log(&self, record: &mut FastLogRecord) {
            let data;
            match record.level {
                Level::Warn | Level::Error => {
                    data = format!(
                        "{} {} {} - {}  {}\n",
                        &record.now,
                        record.level,
                        record.module_path,
                        record.args,
                        record.format_line()
                    );
                }
                _ => {
                    data = format!(
                        "{} {} {} - {}\n",
                        &record.now, record.level, record.module_path, record.args
                    );
                }
            }
            print!("{}", data);
    }
}

fn main(){
    fast_log::init_custom_log(
        vec![Box::new(CustomLog {})],
        1000,
        log::Level::Info,
        Box::new(NoFilter {}),
        Box::new(FastLogFormatRecord {}),
    );
    log::info!("Commencing yak shaving");
    log::error!("Commencing error");
    sleep(Duration::from_secs(1));
}