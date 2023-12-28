use fast_log::appender::{FastLogRecord, LogAppender};
use fast_log::config::Config;
use fastdate::DateTime;
use log::Level;

struct CustomLog {}

impl LogAppender for CustomLog {
    fn do_logs(&self, records: &[FastLogRecord]) {
        for record in records {
            let now = DateTime::from(record.now);
            let data;
            match record.level {
                Level::Warn | Level::Error => {
                    data = format!(
                        "{} {} {} - {}  {}\n",
                        now, record.level, record.module_path, record.args, record.formated
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
}

fn main() {
    fast_log::init(Config::new().custom(CustomLog {})).unwrap();
    log::info!("Commencing yak shaving");
    log::error!("Commencing error");
    log::logger().flush();
}
