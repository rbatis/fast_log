# log
#[中文](README_CH.md)

[![Build Status](https://app.travis-ci.com/rbatis/fast_log.svg?branch=master)](https://app.travis-ci.com/rbatis/fast_log)
[![GitHub release](https://img.shields.io/github/v/release/rbatis/fast_log)](https://github.com/rbatis/fast_log/releases)


the fast log . This crate uses #! [forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.

A log implementation for extreme speed, using Crossbeam to double the efficiency (as opposed to the standard library MPSC) with a lockless channel, using a channel to write logs asynchronously. Completely use the safe code to achieve, without safe code


* Low overhead, based on May coroutines

* High performance, use lockless message queue, log is stored in queue, then flush disk. It does not block the caller

* Full APPEND mode file writing, high efficiency for solid state/mechanical disk (solid state and mechanical disk sequential write performance is better than random write)

* Built-in ZIP compression, compressed file name date + serial number, no need to worry about the log file is too large

* Built-in log segmentation, custom log full number of immediately split

* Built-in filtering configuration support, can be customized to filter out other library printed logs

* Support custom compression algorithms, such as ZIP and LZ4




```

              -----------------
log data->    | main channel(crossbeam)  |   ->          
              ----------------- 
                                        ----------------                                    ----------------------
                                  ->    |coroutines channel(may)|  -> background coroutines  |    appenders  |
                                        ----------------                                    ----------------------


```



> How fast is >?

// Win10 (PC 6 core, mechanical hard disk)

* Use QPS: 525892 /s



// Win10 (PC 6 core, SSD)

* USE QPS: 508215 pieces /s



> support Future mode,async await based on mpsc channel, tokio or async_std
> support log split,zip_compress

* how to use?

```toml
log = "0.4"
#default is enable zip packer
fast_log = {version = "1.4"}
```
or
```toml
log = "0.4"
#default is enable zip packer,this is allow lz4 packer(this is vary faster)
fast_log = {version = "1.4" , features = ["lz4"]}
```





#### Use Log

```rust
use fast_log::{init_log};
use log::{error, info, warn};
fn  main(){
    fast_log::init_log("requests.log",  log::Level::Info, None,true);
    info!("Commencing yak shaving");
}
```



##### split log, allow_zip_compress = Zip compression

```rust
#[test]
pub fn test_file_compation() {
    init_split_log("target/logs/",  LogSize::MB(1), false, log::Level::Info, None, Box::new(ZipPacker{}), true);// or Box::new(LZ4Packer{})
    for _ in 0..200000 {
        info!("Commencing yak shaving");
    }
    may::coroutine::sleep(Duration::from_secs(1));
}
```



##### Custom Log

```rust
use fast_log::{init_custom_log,LogAppender};
use log::{error, info, warn};

pub struct CustomLog{}
impl LogAppender for CustomLog{
    fn do_log(&mut self, record: &FastLogRecord) {
        print!("{}",record);
    }
}
fn  main(){
    fast_log::init_custom_log(vec![Box::new(CustomLog {})],  log::Level::Info, Box::new(NoFilter {}));
    info!("Commencing yak shaving");
    may::coroutine::sleep(std::time::Duration::from_secs(1));
}
```
