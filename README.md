# log
the fast log  . This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.
一款追求极致速度的日志实现，使用crossbeam 无锁channel提高一倍效率(相对于标准库的mpsc)，使用channel异步写日志。完全使用safe 代码实现，无不安全代码

* 使用无锁消息队列,日志先存于队列中，后续flush磁盘。不阻塞调用方
* 全Append模式写入文件，对固态/机械磁盘效率高


* 有多快？
//win10(PC 6核心,机械硬盘)
* use QPS: 525892 条/s
//win10(PC 6核心,固态硬盘)
* use QPS: 508215 条/s

* support Future mode,async await based on mpsc channel, tokio or async_std
* how to use?
```toml
log = "0.4"
fast_log="1.3"
```


#### use log 使用日志
```rust
use fast_log::{init_log};
use log::{error, info, warn};
fn  main(){
    init_log("requests.log", 1000, log::Level::Info);
    info!("Commencing yak shaving");
}
```

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
    init_custom_log(vec![Box::new(custom_log)],1000, log::Level::Info,true);
    info!("Commencing yak shaving");
    std::thread::sleep(std::time::Duration::from_secs(1));
}
```

