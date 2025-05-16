use crate::appender::{FastLogRecord, LogAppender};

/// only write append into console
pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        if records.len() == 0 {
            return;
        }
        let cap = records.iter().map(|record| record.formated.len()).sum();
        let mut buffer = String::with_capacity(cap);
        for x in records {
            buffer.push_str(&x.formated);
        }
        print!("{}", buffer);
    }
}

/// only write append to stderr
pub struct ConsoleStderrAppender {}

impl LogAppender for ConsoleStderrAppender {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        if records.len() == 0 {
            return;
        }
        let cap = records.iter().map(|record| record.formated.len()).sum();
        let mut buffer = String::with_capacity(cap);
        for record in records {
            buffer.push_str(&record.formated);
        }
        eprint!("{}", buffer);
    }
}
