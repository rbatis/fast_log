use crate::fast_log::LogAppender;

pub struct ConsoleAppender {}

impl LogAppender for ConsoleAppender{
    fn do_log(&mut self, info: &str) {
        print!("{}",info);
    }
}