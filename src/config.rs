use crate::appender::{LogAppender, RecordFormat};
use crate::consts::LogSize;
use crate::filter::Filter;
use crate::plugin::console::{ConsoleAppender, ConsoleStderrAppender};
use crate::plugin::file::FileAppender;
use crate::plugin::file_loop::FileLoopAppender;
use crate::plugin::file_split::{
    CanRollingPack, FileSplitAppender, Keep, Packer, RawFile, SplitFile,
};
use crate::FastLogFormat;
use arc_swap::ArcSwap;
use log::LevelFilter;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

/// the fast_log Config
/// for example:
/// ```rust
/// use fast_log::Config;
/// fn main(){
///    fast_log::init(Config::new().console().chan_len(Some(1000000))).unwrap();
/// }
/// ```
pub struct Config {
    /// Each appender is responsible for printing its own business
    /// every LogAppender have one thread(need Mutex) access this.
    /// copy-on-write, support dynamic append after init
    pub appends: ArcSwap<Vec<Arc<Mutex<Box<dyn LogAppender>>>>>,
    /// the log level filter
    pub level: LevelFilter,
    /// filter log (copy-on-write, support dynamic append after init)
    pub filters: ArcSwap<Vec<Arc<dyn Filter>>>,
    /// format record into field fast_log_record's formatted:String
    pub format: Box<dyn RecordFormat>,
    /// the channel length,default None(Unbounded channel)
    pub chan_len: Option<usize>,
    /// number of worker threads that receive and dispatch log records from the main channel to appenders
    pub worker_tasks: Option<usize>,
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("appends", &self.appends.load().len())
            .field("filters", &self.filters.load().len())
            .field("level", &self.level)
            .field("chan_len", &self.chan_len)
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appends: ArcSwap::from_pointee(Vec::new()),
            level: LevelFilter::Trace,
            filters: ArcSwap::from_pointee(Vec::new()),
            format: Box::new(FastLogFormat::new()),
            chan_len: None,
            worker_tasks: Some(1),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// copy-on-write push appender
    fn push_appender(self, appender: Box<dyn LogAppender>) -> Self {
        let mut nv = self.appends.load().as_ref().clone();
        nv.push(Arc::new(Mutex::new(appender)));
        self.appends.store(Arc::new(nv));
        self
    }

    /// copy-on-write push filter
    fn push_filter(self, filter: Box<dyn Filter>) -> Self {
        let mut nv = self.filters.load().as_ref().clone();
        nv.push(Arc::from(filter));
        self.filters.store(Arc::new(nv));
        self
    }

    /// set log LevelFilter
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }
    /// add log Filter
    pub fn add_filter<F: Filter + 'static>(self, filter: F) -> Self {
        self.push_filter(Box::new(filter))
    }

    /// add log Filter
    pub fn filter(self, filters: Vec<Box<dyn Filter>>) -> Self {
        let mut nv = self.filters.load().as_ref().clone();
        nv.extend(filters.into_iter().map(Arc::from));
        self.filters.store(Arc::new(nv));
        self
    }
    /// set log format
    pub fn format<F: RecordFormat + 'static>(mut self, format: F) -> Self {
        self.format = Box::new(format);
        self
    }
    /// add a ConsoleAppender
    pub fn console(self) -> Self {
        self.push_appender(Box::new(ConsoleAppender {}))
    }
    /// add a ConsoleStderrAppender
    pub fn console_stderr(self) -> Self {
        self.push_appender(Box::new(ConsoleStderrAppender {}))
    }
    /// add a FileAppender
    pub fn file(self, file: &str) -> Self {
        self.push_appender(Box::new(FileAppender::new(file).unwrap()))
    }
    /// add a FileLoopAppender
    pub fn file_loop(self, file: &str, max_temp_size: LogSize) -> Self {
        self.push_appender(Box::new(
            FileLoopAppender::new(file, max_temp_size).expect("make file_loop fail"),
        ))
    }
    /// add a FileSplitAppender
    pub fn file_split<
        R: CanRollingPack + 'static,
        K: Keep + 'static,
        P: Packer + Sync + 'static,
    >(
        self,
        file_path: &str,
        rolling: R,
        keeper: K,
        packer: P,
    ) -> Self {
        self.push_appender(Box::new(
            FileSplitAppender::new::<RawFile>(
                file_path,
                Box::new(rolling),
                Box::new(keeper),
                Box::new(packer),
            )
            .expect("new split file fail"),
        ))
    }

    /// add a SplitAppender
    /// .split::<FileType, Packer>()
    /// for example:
    /// ```rust
    /// use fast_log::Config;
    /// use fast_log::consts::LogSize;
    /// use fast_log::plugin::file_split::{Rolling, RawFile, RollingType, KeepType};
    /// use fast_log::plugin::packer::LogPacker;
    /// fn new(){
    ///  fast_log::init(
    ///         Config::new()
    ///             .chan_len(Some(100000))
    ///             .split::<RawFile, _, _, _>(
    ///                 "target/logs/temp.log",
    ///                 KeepType::All,
    ///                 LogPacker {},
    ///                 Rolling::new(RollingType::BySize(LogSize::MB(1))),
    ///             ),
    ///     );
    /// }
    /// ```
    pub fn split<
        F: SplitFile + 'static,
        R: Keep + 'static,
        P: Packer + Sync + 'static,
        H: CanRollingPack + 'static,
    >(
        self,
        file_path: &str,
        keeper: R,
        packer: P,
        how_pack: H,
    ) -> Self {
        self.push_appender(Box::new(
            FileSplitAppender::new::<F>(
                file_path,
                Box::new(how_pack),
                Box::new(keeper),
                Box::new(packer),
            )
            .expect("new split file fail"),
        ))
    }
    /// add a custom LogAppender
    pub fn custom<Appender: LogAppender + 'static>(self, arg: Appender) -> Self {
        self.add_appender(arg)
    }

    /// add a LogAppender
    pub fn add_appender<Appender: LogAppender + 'static>(self, arg: Appender) -> Self {
        self.push_appender(Box::new(arg))
    }

    /// if none=> unbounded() channel,if Some =>  bounded(len) channel
    pub fn chan_len(mut self, len: Option<usize>) -> Self {
        self.chan_len = len;
        self
    }

    /// set the number of worker threads, default is 1
    pub fn worker_tasks(mut self, num: Option<usize>) -> Self {
        self.worker_tasks = num;
        self
    }
}