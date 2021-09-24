use crate::appender::{FastLogRecord, LogAppender};

/// only write append into console
pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender {
    fn do_log(&self, record: &mut FastLogRecord) {
        print!("{}", record.formated);
    }
}
