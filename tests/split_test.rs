#[cfg(test)]
mod test {
    use fast_log::appender::{Command, FastLogRecord, LogAppender};
    use fast_log::consts::LogSize;
    use fast_log::plugin::file_split::{FileSplitAppender, Packer, RawFile, Rolling};
    use fast_log::plugin::packer::LogPacker;
    use fast_log::plugin::rolling::{RollingAll, RollingNum};
    use log::Level;
    use std::fs::remove_dir_all;
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_send_pack() {
        let _ = remove_dir_all("target/test/");
        let appender = FileSplitAppender::<RawFile, LogPacker>::new(
            "target/test/",
            LogSize::MB(1),
            RollingAll {},
            LogPacker {},
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
        let rolling_num = RollingNum { num: 0 }.do_rolling("target/test/", "temp.log");
        assert_eq!(rolling_num, 1);
        let _ = remove_dir_all("target/test/");
    }

    #[test]
    fn test_parse_log_name() {
        let t = LogPacker {}
            .parse_log_name("temp2023-07-20T10-13-17.452247.log", "temp.log")
            .unwrap();
        assert_eq!(t.to_string(), "2023-07-20 10:13:17.452247");
    }
}
