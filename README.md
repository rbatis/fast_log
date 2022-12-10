# fast_log

[![Build Status](https://img.shields.io/github/workflow/status/rbatis/fast_log/Rust)](https://github.com/rbatis/fast_log/actions)
[![GitHub release](https://img.shields.io/github/v/release/rbatis/fast_log)](https://github.com/rbatis/fast_log/releases)


the fast log . This crate uses #! [forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.

A log implementation for extreme speed, using Crossbeam/channel ,once Batch write logs,fast log date, Appender architecture, appender per thread

* Low overhead, log write based on thread, also support tokio/Future

* High performance, use lockless message queue, log is stored in queue, then flush disk. It does not block the caller

* Full APPEND mode file writing, high efficiency for solid state/mechanical disk (solid state and mechanical disk sequential write performance is better than random write)

* When channel pressure increases, logs can be written in batches at a time

* Built-in ZIP compression, compressed file name date + serial number, no need to worry about the log file is too large

* Built-in log segmentation, custom log full number of immediately split

* Built-in filtering configuration support, can be customized to filter out other library printed logs

* Support custom compression algorithms, such as ZIP and LZ4

* Support use ```log::logger().flush()``` method wait to flush disk

* Simple and efficient Appender architecture.Both configuration and customization are simple


```

              -----------------
log data->    | main channel(crossbeam)  |   ->          
              ----------------- 
                                        ----------------                                    ----------------------
                                  ->    |thread channel)|  -> background thread  |    appender1  |
                                        ----------------                                    ----------------------

                                        ----------------                                    ----------------------
                                  ->    |thread channel)|  -> background thread  |    appender2  |
                                        ----------------                                    ----------------------

                                        ----------------                                    ----------------------
                                  ->    |thread channel)|  -> background thread  |    appender3  |
                                        ----------------                                    ----------------------

                                        ----------------                                    ----------------------
                                  ->    |thread channel)|  -> background thread  |    appender4  |
                                        ----------------                                    ----------------------


```

* How fast is?

* no flush(chan_len=1000000) benches/log.rs
```
//MACOS(Apple M1MAX-32GB)
test bench_log ... bench:          85 ns/iter (+/- 1,800)
```

* all log flush into file(chan_len=1000000) example/bench_test_file.rs
```
//MACOS(Apple M1MAX-32GB)
test bench_log ... bench:          323 ns/iter (+/- 0)
```

* how to use?

```toml
log = "0.4"
fast_log = {version = "1.5"}
```
or enable zip/lz4/gzip Compression library
```toml
log = "0.4"
# "lz4","zip","gzip"
fast_log = {version = "1.5" , features = ["lz4","zip","gzip"]}
```

#### Performance optimization(important)

* use ```chan_len(Some(100000))``` Preallocating channel memory reduces the overhead of memory allocation，for example:

```rust
use log::{error, info, warn};
fn  main(){
    fast_log::init(Config::new().file("target/test.log").chan_len(Some(100000))).unwrap();
    log::info!("Commencing yak shaving{}", 0);
}
```


#### Use Log(Console)

```rust
use log::{error, info, warn};
fn  main(){
    fast_log::init(Config::new().console().chan_len(Some(100000))).unwrap();
    log::info!("Commencing yak shaving{}", 0);
}
```

#### Use Log(Console Print)

```rust
use log::{error, info, warn};
fn  main(){
    fast_log::init(Config::new().console().chan_len(Some(100000))).unwrap();
    fast_log::print("Commencing print\n".into());
}
```

#### Use Log(File)

```rust
use fast_log::{init_log};
use log::{error, info, warn};
fn  main(){
    fast_log::init(Config::new().file("target/test.log").chan_len(Some(100000))).unwrap();
    log::info!("Commencing yak shaving{}", 0);
    info!("Commencing yak shaving");
}
```



#### Split Log(.log packer)

```rust
use fast_log::plugin::file_split::RollingType;
use fast_log::consts::LogSize;
use fast_log::plugin::packer::LogPacker;

#[test]
pub fn test_file_compation() {
    fast_log::init(Config::new()
        .console()
        .chan_len(Some(100000))
        .file_split("target/logs/",
                    LogSize::MB(1),
                    RollingType::All,
                    LogPacker{})).unwrap();
    for _ in 0..200000 {
        info!("Commencing yak shaving");
    }
    log::logger().flush();
}
```



##### Custom Log(impl do_log method)

```rust
use fast_log::{LogAppender};
use log::{error, info, warn};

pub struct CustomLog{}
impl LogAppender for CustomLog{
    fn do_log(&mut self, record: &FastLogRecord) {
        print!("{}",record);
    }
}
fn  main(){
    let wait = fast_log::init(Config::new().custom(CustomLog {}).chan_len(Some(100000))).unwrap();
    info!("Commencing yak shaving");
    log::logger().flush();
}
```
