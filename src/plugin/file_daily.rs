use crate::appender::{Command, FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;
use crate::plugin::file_split::{self, LogPack, Packer, SplitFile};
use crate::plugin::roller::Roller;
use crate::{chan, Sender};
use fastdate::{DateTime, DurationFrom};
use std::cell::RefCell;
use std::io::SeekFrom;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// split log file allow pack compress log
/// Memory space swop running time , reduces the number of repeated queries for IO
pub struct FileDailyAppender<F: SplitFile> {
    dir_path: String,
    file: RefCell<F>,
    sender: Sender<LogPack>,
    temp_size: LogSize,
    roller: Box<dyn Roller>,
    //cache data
    temp_bytes: AtomicUsize,
    temp_name: String,
    temp_date: RefCell<DateTime>,
    roll_date: RefCell<DateTime>,
    temp_file_index: AtomicUsize,
}

impl<F: SplitFile> FileDailyAppender<F> {
    pub fn new(
        file_path: &str,
        temp_size: LogSize,
        roller: Box<dyn Roller>,
        packer: Box<dyn Packer>,
    ) -> Result<FileDailyAppender<F>, LogError> {
        let temp_name = {
            let buf = Path::new(&file_path);
            match buf.extension().unwrap_or_default().to_str().unwrap() {
                "txt" | "log" => buf
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
                _ => "temp.log".to_string(),
            }
        };
        let mut dir_path = file_path.trim_end_matches(&temp_name).to_string();
        if dir_path.is_empty() {
            if let Ok(v) = std::env::current_dir() {
                dir_path = v.to_str().unwrap_or_default().to_string();
            }
        }
        std::fs::create_dir_all(&dir_path);
        let path = Path::new(&dir_path);
        let temp_date = DateTime::now()
            .set_hour(0)
            .set_min(0)
            .set_sec(0)
            .set_micro(0);
        let roll_date = temp_date.clone().add(Duration::from_day(1));
        let temp_file_index = AtomicUsize::new(0);
        let mut pack_not_found = false;
        for i in 0..usize::MAX {
            let name = path.join(calc_filename(&temp_name, i + 1, &temp_date));
            let packed_name = name.with_extension(packer.pack_name());
            if !pack_not_found {
                match std::fs::metadata(packed_name) {
                    Ok(_) => continue,
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            temp_file_index.store(i, Ordering::Relaxed);
                            pack_not_found = true;
                        }
                        _ => panic!("{:?}", e),
                    },
                }
            }
            if pack_not_found {
                match std::fs::metadata(name) {
                    Ok(_) => continue,
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            temp_file_index.store(i, Ordering::Relaxed);
                            break;
                        }
                        _ => panic!("{:?}", e),
                    },
                }
            }
        }
        let temp_file = path.join(calc_filename(
            &temp_name,
            temp_file_index.load(Ordering::Relaxed),
            &temp_date,
        ));
        let temp_bytes = AtomicUsize::new(0);
        let file = F::new(temp_file.to_str().unwrap(), temp_size)?;
        let mut offset = file.offset();
        if offset != 0 {
            offset += 1;
        }
        temp_bytes.store(offset, Ordering::Relaxed);
        file.seek(SeekFrom::Start(temp_bytes.load(Ordering::Relaxed) as u64));
        let (sender, receiver) = chan(None);
        file_split::spawn_saver(temp_name.clone(), receiver, packer);
        Ok(Self {
            temp_bytes,
            dir_path: dir_path.to_string(),
            file: RefCell::new(file),
            sender,
            temp_size,
            temp_name,
            roller,
            temp_date: RefCell::new(temp_date),
            roll_date: RefCell::new(roll_date),
            temp_file_index,
        })
    }

    /// send data make an pack,and truncate data when finish.
    fn send_pack(&self, day_changed: bool, record_date: DateTime) {
        let current_filename = Path::new(&self.dir_path).join(self.current_temp_filename());
        if day_changed {
            *self.temp_date.borrow_mut() = record_date.clone();
            *self.roll_date.borrow_mut() = record_date.clone().add(Duration::from_day(1));
            self.temp_file_index.store(0, Ordering::SeqCst);
        } else {
            self.temp_file_index.fetch_add(1, Ordering::SeqCst);
        }
        self.truncate();

        self.sender.send(LogPack {
            dir: self.dir_path.clone(),
            rolling: dyn_clone::clone_box(&*self.roller),
            new_log_name: current_filename.to_str().unwrap().to_string(),
        });
    }

    fn current_temp_filename(&self) -> String {
        calc_filename(
            &self.temp_name,
            self.temp_file_index.load(Ordering::Relaxed),
            &self.temp_date.borrow(),
        )
    }

    pub fn truncate(&self) {
        // flush data
        self.file.borrow().flush();
        // create new file
        let temp_file = Path::new(&self.dir_path).join(self.current_temp_filename());
        match F::new(temp_file.to_str().unwrap(), self.temp_size) {
            Ok(f) => {
                *self.file.borrow_mut() = f;
                self.temp_bytes.store(0, Ordering::SeqCst);
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn calc_filename(name: &str, index: usize, date: &DateTime) -> String {
    let (name, ext) = name.split_at(name.rfind(".").unwrap());
    format!(
        "{name}_{}_{index}{ext}",
        date.to_string()[0..10].replace("-", "")
    )
}

impl<F: SplitFile> LogAppender for FileDailyAppender<F> {
    fn do_logs(&self, records: &[FastLogRecord]) {
        //if temp_bytes is full or day changed, must send pack
        let mut temp = String::with_capacity(records.len() * 10);
        for x in records {
            match x.command {
                Command::CommandRecord => {
                    let record_date = fastdate::DateTime::from(x.now)
                        .set_hour(0)
                        .set_min(0)
                        .set_sec(0)
                        .set_micro(0);
                    let day_changed = record_date >= *self.roll_date.borrow();
                    if (self.temp_bytes.load(Ordering::Relaxed)
                        + temp.as_bytes().len()
                        + x.formated.as_bytes().len())
                        >= self.temp_size.get_len()
                        || day_changed
                    {
                        self.temp_bytes.fetch_add(
                            {
                                let w = self.file.borrow().write(temp.as_bytes());
                                if let Ok(w) = w {
                                    w
                                } else {
                                    0
                                }
                            },
                            Ordering::SeqCst,
                        );
                        temp.clear();
                        self.send_pack(day_changed, record_date);
                    }
                    temp.push_str(x.formated.as_str());
                }
                Command::CommandExit => {}
                Command::CommandFlush(_) => {}
            }
        }
        if !temp.is_empty() {
            self.temp_bytes.fetch_add(
                {
                    let w = self.file.borrow().write(temp.as_bytes());
                    if let Ok(w) = w {
                        w
                    } else {
                        0
                    }
                },
                Ordering::SeqCst,
            );
        }
    }

    fn flush(&self) {}
}
