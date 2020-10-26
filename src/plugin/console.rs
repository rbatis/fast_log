use crate::fast_log::LogAppender;

/// only write append into console
pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender{
    fn do_log(&mut self, info: &str) {
        print!("{}",info);
    }
}