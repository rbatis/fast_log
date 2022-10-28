#[cfg(test)]
mod test {
    use fast_log::appender::{Command, FastLogRecord, LogAppender};
    use fast_log::consts::LogSize;
    use fast_log::plugin::file_split::{FileSplitAppender, RollingType};
    use fast_log::plugin::packer::LogPacker;
    use log::Level;
    use std::fs::remove_dir_all;
    use std::time::SystemTime;

    #[test]
    fn test_send_pack() {
        let _ = remove_dir_all("target/test/");
        let appender = FileSplitAppender::new(
            "target/test/",
            LogSize::MB(1),
            RollingType::All,
            Box::new(LogPacker {}),
        );
        appender.do_logs(&[FastLogRecord {
            command: Command::CommandRecord,
            level: Level::Error,
            target: "".to_string(),
            args: "".to_string(),
            module_path: "".to_string(),
            file: "".to_string(),
            line: None,
            now: SystemTime::now(),
            formated: "".to_string(),
        }]);
        appender.cell.borrow_mut().send_pack();
        let rolling_num = RollingType::KeepNum(0).do_rolling("temp", "target/test/");
        assert_eq!(rolling_num, 1);
        let _ = remove_dir_all("target/test/");
    }
}
