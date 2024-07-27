use crate::appender::{FastLogRecord, LogAppender};

/// only write append into console
pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        if records.len() == 0 {
            return;
        }
        let mut cap = 0;
        if records.len() != 0 {
            cap = 0;
            for x in records {
                cap += x.formated.len();
            }
        }
        let mut buffer = String::with_capacity(cap);
        for x in records {
            buffer.push_str(&x.formated);
        }
        print!("{}", buffer);
    }
}
