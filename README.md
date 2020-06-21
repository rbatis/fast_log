# log
the fast log  . This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.

* support Future mode,async await based on mpsc channel, tokio or async_std
* how to use?
```toml
log = "0.4"
fast_log="*"
```


#### use log
```rust
use log::{error, info, warn};
fn  main(){
    fast_log::log::init_log("requests.log", &RuntimeType::Std);
    info!("Commencing yak shaving");
}
```

