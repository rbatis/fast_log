# log
the fast log

* how to use?
```toml
log = "0.4"
fast_log="1.0.0"
```

```rust
use log::{error, info, warn};
fn  main(){
    fast_log::log::init_log("requests.log");
    info!("Commencing yak shaving");
}
```
