#![feature(test)]
extern crate test;

use fast_log::Config;

use test::{black_box, Bencher};

// 85 ns/iter (+/- 2073)
#[bench]
fn bench_log_file(b: &mut Bencher) {
    let _ = std::fs::remove_file("target/test_bench.log");
    fast_log::init(
        Config::new()
            .file("target/test_bench.log")
            .chan_len(Some(1000000)),
    )
    .unwrap();
    b.iter(|| {
        black_box({
            log::info!("Commencing yak shaving");
        });
    });
}
