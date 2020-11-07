use crate::fast_log::{LogAppender, FastLogRecord};

/// only write append into console
pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender {
    fn do_log(&self, record: &FastLogRecord){
        print!("{}", record.formated);
    }
}