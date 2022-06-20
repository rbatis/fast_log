use fast_log::appender::{LogAppender, FastLogRecord};
use log::Level;
use chrono::{DateTime, Local};
use tokio::runtime::Runtime;
use fast_log::config::Config;

struct CustomLog {
    rt:Runtime
}

impl LogAppender for CustomLog {
    fn do_logs(&self, records: &[FastLogRecord]) {
        let mut datas = String::new();
        for record in records {
            let now: DateTime<Local> = chrono::DateTime::from(record.now);
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
            datas.push_str(&data);
        }
        self.rt.block_on(async move {
            //send to web,file,any way
            print!("{}", datas);
        });
    }
}

#[tokio::main]
async fn main() {
     fast_log::init(Config::new().custom(CustomLog {
        rt: tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap()
    })).unwrap();
    log::info!("Commencing yak shaving");
    log::error!("Commencing error");
    log::logger().flush();
}