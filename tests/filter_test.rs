#[cfg(test)]
mod test {
    use log::LevelFilter;
    use fast_log::{Config, FastLogFormat};
    use fast_log::appender::{FastLogRecord, LogAppender};
    use fast_log::filter::ModuleFilter;

    #[test]
    fn test_send_pack() {
        let m = ModuleFilter::new();
        m.add( module_path!());
        pub struct A{}
        impl LogAppender for A{
            fn do_logs(&self, _records: &[FastLogRecord]) {
                panic!("must be filter log,but do_log");
            }
        }
        fast_log::init(Config::new()
            .console()
            .format(FastLogFormat::new().set_display_line_level(LevelFilter::Trace))
            .add_filter(m)).unwrap();
        log::info!("aaa");
        log::logger().flush();
    }
}