use std::time::Duration;
use fast_log::appender::{FastLogFormatRecord, LogAppender, FastLogRecord};
use fast_log::filter::NoFilter;
use log::Level;
use std::thread::sleep;
use chrono::{DateTime, Local};

struct CustomLog {}

impl LogAppender for CustomLog {
    fn do_log(&self, record: &FastLogRecord) {
            let now:DateTime<Local> = chrono::DateTime::from(record.now);
            let data;
            match record.level {
                Level::Warn | Level::Error => {
                    data = format!(
                        "{} {} {} - {}  {}\n",
                        now,
                        record.level,
                        record.module_path,
                        record.args,
                        record.formated
                    );
                }
                _ => {
                    data = format!(
                        "{} {} {} - {}\n",
                        &now, record.level, record.module_path, record.args
                    );
                }
            }
            print!("{}", data);
    }
}

fn main(){
    let wait=fast_log::init_custom_log(
        vec![Box::new(CustomLog {})],
        log::Level::Info,
        Box::new(NoFilter {}),
        Box::new(FastLogFormatRecord::new()),
    ).unwrap();
    log::info!("Commencing yak shaving");
    log::error!("Commencing error");
    wait.wait();
}