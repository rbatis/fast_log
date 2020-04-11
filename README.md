# log
the fast log  . This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% Safe Rust.

* how to use?
```toml
log = "0.4"
fast_log="*"
```

```rust
use log::{error, info, warn};
fn  main(){
    fast_log::log::init_log("requests.log");
    info!("Commencing yak shaving");
}
```


* or use tokio
```rust
#[tokio::main]
#[test]
async fn bench_async_log() {
    init_async_log("requests.log").await;
}
```
