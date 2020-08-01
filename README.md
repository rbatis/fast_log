# log
the fast log  . This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.
一款追求极致速度的日志实现，使用crossbeam 无锁channel提高一倍效率(相对于标准库的mpsc)，使用channel异步写日志。完全使用safe 代码实现，无不安全代码


* support Future mode,async await based on mpsc channel, tokio or async_std
* how to use?
```toml
log = "0.4"
fast_log="1.2.3"
```


#### use log 使用日志
```rust
use log::{error, info, warn};
fn  main(){
    fast_log::log::init_log("requests.log", &RuntimeType::Std);
    info!("Commencing yak shaving");
}
```

