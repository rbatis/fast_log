use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use fast_log::appender::{FastLogFormatRecord, LogAppender, FastLogRecord};
use fast_log::filter::NoFilter;
use log::Level;
use std::thread::sleep;
use chrono::{DateTime, Local};
use meilisearch_sdk::client::Client;
use meilisearch_sdk::document::Document;
use meilisearch_sdk::indexes::Index;
use tokio::runtime::Runtime;
use fast_log::config::Config;
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct LogDoc {
    id: usize,
    log: String,
}

// That trait is required to make a struct usable by an index
impl Document for LogDoc {
    type UIDType = usize;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}


struct CustomLog {
    c:Arc<Index>,
    rt:Runtime
}

impl LogAppender for CustomLog {
    fn do_log(&self, record: &FastLogRecord) {
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
        let id = now.timestamp_millis() as usize;
        let c=self.c.clone();
        self.rt.block_on(async move {
            //send to web,file,any way
            let log=LogDoc{
                id: id,
                log: data.to_string(),
            };
            c.add_documents(&[log], Some("id")).await.unwrap();
            print!("{}", data);
        });
    }
}

/// you should download(https://github.com/meilisearch/meilisearch-rust/releases) and run meilisearch
#[tokio::main]
async fn main() {
    let client = Client::new("http://localhost:7700", "masterKey");
    let doc= client.index("LogDoc");
    let wait = fast_log::init(Config::new().custom(CustomLog {
        c:Arc::new(doc),
        rt: tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap()
    })).unwrap();
    for _ in 0..1000{
        log::info!("Commencing yak shaving");
        log::error!("Commencing error");
    }
    wait.wait();
}