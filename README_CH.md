# log
the fast log  . This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.

[![Build Status](https://app.travis-ci.com/rbatis/fast_log.svg?branch=master)](https://app.travis-ci.com/rbatis/fast_log)


一款追求极致速度的日志实现，使用crossbeam 无锁channel提高一倍效率(相对于标准库的mpsc)，使用channel异步写日志。完全使用safe 代码实现，无不安全代码

* 低开销，基于线程
* 高性能，使用无锁消息队列,日志先存于队列中，后续flush磁盘。不阻塞调用方
* 全Append模式写入文件，对固态/机械磁盘效率高（固态以及机械硬盘 顺序写性能好于随机写）
* 在通道压力变大时，支持批量一次性写入日志
* 内置 Zip压缩，压缩文件名为日期+序号，无需操心日志文件过大
* 内置 日志分割，自定义日志满多少条数立即分割
* 内置 过滤配置支持，可自定义过滤掉其他库打印的日志


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







> 有多快？

//win10(PC 6核心,机械硬盘)
* QPS: 1127002 条/s

//win10(PC 6核心,固态硬盘)
* QPS: 1092635 条/s

> support Future mode,async await based on mpsc channel, tokio or async_std
> support log split,zip_compress

* how to use?
```toml
log = "0.4"
fast_log="1.5"
```


#### use log 简单日志
```rust
use log::{error, info, warn};
fn  main(){
    fast_log::init(Config::new().console()).unwrap();
    log::info!("Commencing yak shaving{}", 0);
}
```

##### split log 分割日志
```rust
use fast_log::plugin::file_split::RollingType;
use fast_log::consts::LogSize;
use fast_log::plugin::packer::LogPacker;

#[test]
pub fn test_file_compation() {
    fast_log::init(Config::new()
        .console()
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

##### custom log 自定义日志
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
    let wait = fast_log::init(Config::new().custom(CustomLog {})).unwrap();
    info!("Commencing yak shaving");
    log::logger().flush();
}
```

