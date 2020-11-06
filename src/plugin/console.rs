use crate::fast_log::{LogAppender, FastLogRecord};
use log::Level;

/// only write append into console
pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender {
    fn do_log(&mut self, record: &FastLogRecord) {
        let data;
        match record.level {
            Level::Warn | Level::Error => {
                data = format!("{} {} {} - {}  {}\n", &record.now, record.level, record.module_path, record.args, record.format_line());
            }
            _ => {
                data = format!("{} {} {} - {}\n", &record.now, record.level, record.module_path, record.args);
            }
        }
        print!("{}", data);
    }
}