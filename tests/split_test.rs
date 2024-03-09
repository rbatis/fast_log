#[cfg(test)]
mod test {
    use fast_log::appender::{Command, FastLogRecord, LogAppender};
    use fast_log::consts::LogSize;
    use fast_log::plugin::file_name::FileName;
    use fast_log::plugin::file_split::{FileSplitAppender, Keep, Packer, RawFile, RollingType};
    use fast_log::plugin::packer::LogPacker;
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
            RollingType::All,
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
        let rolling_num = RollingType::KeepNum(0).do_keep("target/test/", "temp.log");
        assert_eq!(rolling_num, 1);
        let _ = remove_dir_all("target/test/");
    }

    #[test]
    fn test_log_name_create() {
        let p = LogPacker {};
        let name = p.log_name_create("temp.log");
        assert_eq!(name.ends_with(".log"), true);
    }

    #[test]
    fn test_extract_file_name() {
        let p = "temp.log".extract_file_name();
        assert_eq!(p, "temp.log");
    }

    #[test]
    fn test_extract_file_name2() {
        let p = "logs/temp.log".extract_file_name();
        assert_eq!(p, "temp.log");
    }

    #[test]
    fn test_extract_file_name3() {
        let p = "logs/".extract_file_name();
        assert_eq!(p, "");
    }

    #[test]
    fn test_extract_file_name4() {
        let p = "C:/logs".extract_file_name();
        assert_eq!(p, "logs");
    }

    #[test]
    fn test_extract_file_name5() {
        let p = "C:/logs/aa.log".extract_file_name();
        assert_eq!(p, "aa.log");
    }

    #[test]
    fn test_extract_file_name6() {
        let p = "C:\\logs\\aa.log".extract_file_name();
        assert_eq!(p, "aa.log");
    }
}
