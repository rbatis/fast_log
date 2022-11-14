use crate::appender::{FastLogRecord, LogAppender};

/// only write append into console
pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender {
    fn do_logs(&self, records: &[FastLogRecord]) {
        if records.len() == 0 {
            return;
        }
        let mut buffer = String::with_capacity(records.len());
        for x in records {
            buffer.push_str(&x.formated);
        }
        print!("{}", buffer);
    }
}
