use chrono::{DateTime, Local};
use fast_log::appender::{FastLogFormatRecord, FastLogRecord, LogAppender};
use fast_log::config::Config;
use fast_log::filter::NoFilter;
use log::Level;
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::Runtime;

struct CustomLog {
    rt: Runtime,
}

impl LogAppender for CustomLog {
    fn do_log(&self, record: &FastLogRecord) {
        let now: DateTime<Local> = chrono::DateTime::from(record.now);
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
        self.rt.block_on(async move {
            //send to web,file,any way
            print!("{}", data);
        });
    }
}

#[tokio::main]
async fn main() {
    fast_log::init(
        Config::new().custom(CustomLog {
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        }),
    )
    .unwrap();
    log::info!("Commencing yak shaving");
    log::error!("Commencing error");
    log::logger().flush();
}
