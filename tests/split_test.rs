#[cfg(test)]
mod test {
    use fast_log::appender::{Command, FastLogRecord, LogAppender};
    use fast_log::consts::LogSize;
    use fast_log::plugin::file_split::{FileSplitAppender, RawFile, RollingType};
    use fast_log::plugin::packer::LogPacker;
    use fast_log::plugin::roller::Roller;
    use log::Level;
    use std::fs::remove_dir_all;
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_send_pack() {
        let _ = remove_dir_all("target/test/");
        let appender = FileSplitAppender::<RawFile>::new(
            "target/test/",
            LogSize::MB(1),
            Box::new(RollingType::All),
            Box::new(LogPacker {}),
        )
        .unwrap();
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
        appender.send_pack();
        sleep(Duration::from_secs(1));
        let rolling_num = (Box::new(RollingType::KeepNum(0)) as Box<dyn Roller>)
            .do_rolling("temp.log", "target/test/");
        assert_eq!(rolling_num, 1);
        let _ = remove_dir_all("target/test/");
    }

    #[test]
    fn test_file_name_parse_time() {
        let t = RollingType::file_name_parse_time("temp2023-07-20T10-13-17.452247.log", "temp.log")
            .unwrap();
        assert_eq!(t.to_string(), "2023-07-20 10:13:17.452247");
    }
}
