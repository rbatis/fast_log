#[cfg(test)]
mod test {
    use log::LevelFilter;
    use fast_log::{Config, FastLogFormat};
    use fast_log::appender::{Command, FastLogRecord, LogAppender};
    use fast_log::filter::ModuleFilter;

    #[test]
    fn test_send_pack() {
        let m = ModuleFilter::new();
        m.modules.push(module_path!().to_string());
        pub struct A {}
        impl LogAppender for A {
            fn do_logs(&mut self, records: &[FastLogRecord]) {
                for x in records {
                    if x.command == Command::CommandRecord {
                        panic!("must be filter log,but do_log");
                    }
                }
            }
        }
        fast_log::init(Config::new()
            .format(FastLogFormat::new().set_display_line_level(LevelFilter::Trace))
            .add_filter(m)
            .add_appender(A{})
        ).unwrap();
        log::info!("aaa");
        log::logger().flush();
    }
}
