# fast_log

![Build Status](https://api.travis-ci.com/rbatis/fast_log.svg?branch=master)
[![GitHub release](https://img.shields.io/github/v/release/rbatis/fast_log)](https://github.com/rbatis/fast_log/releases)

<img style="width: 200px;height: 200px;" width="200" height="200" src="https://github.com/rbatis/rbatis/blob/master/logo.png?raw=true" />

A log implementation for extreme speed, using Crossbeam/channel ,once Batch write logs,fast log date, Appender architecture, appender per thread

* High performance,Low overhead, logs auto merge, Full APPEND mode file writing
* Built-in `ZIP`,`LZ4` compression
* Support use ```log::logger().flush()``` method wait to flush disk
* Support custom file(impl Trait)
* Support rolling log(`ByDate`,`BySize`,`ByDuration`)
* Support Keep log(`All`,`KeepTime`,`KeepNum`) Delete old logs,Prevent logs from occupying the disk
* uses `#![forbid(unsafe_code)]` 100% Safe Rust.
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
fast_log = {version = "1.7"}
```
or enable zip/lz4/gzip Compression library
```toml
log = "0.4"
# "lz4","zip","gzip"
fast_log = {version = "1.7" , features = ["lz4","zip","gzip"]}
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


#### Split Log(ByLogDate)

```rust
use fast_log::config::Config;
use fast_log::plugin::file_split::{RollingType, KeepType, DateType, Rolling};
use std::thread::sleep;
use std::time::Duration;
use fast_log::plugin::packer::LogPacker;
fn main() {
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        Rolling::new(RollingType::ByDate(DateType::Day)),
        KeepType::KeepNum(2),
        LogPacker {},
    ))
        .unwrap();
    for _ in 0..60 {
        sleep(Duration::from_secs(1));
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}

```


#### Split Log(ByLogSize)

```rust
use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::{RollingType, KeepType, Rolling};
use fast_log::plugin::packer::LogPacker;
fn main() {
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        Rolling::new(RollingType::BySize(LogSize::KB(500))),
        KeepType::KeepNum(2),
        LogPacker {},
    ))
        .unwrap();
    for _ in 0..40000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
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
