# log
the fast log  . This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.

* support Future mode,async await based on tokio
* support thread mode

* how to use?
```toml
log = "0.4"
fast_log="*"
```



###  use Future mode
```rust
#[tokio::main]
#[test]
async fn bench_async_log() {
    init_async_log("requests.log").await;
    info!("Commencing yak shaving");
}
```

#### use thread mode
```rust
use log::{error, info, warn};
fn  main(){
    fast_log::log::init_log("requests.log");
    info!("Commencing yak shaving");
}
```

