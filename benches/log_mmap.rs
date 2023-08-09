#![feature(test)]
#![feature(bench_black_box)]
extern crate test;

use fast_log::Config;

use fast_log::consts::LogSize;
use fast_log::plugin::file_mmap::MmapFile;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::LogPacker;
use test::{black_box, Bencher};

// 90 ns/iter (+/- 1065)
#[bench]
fn bench_log_mmap(b: &mut Bencher) {
    fast_log::init(
        Config::new()
            .chan_len(Some(100000))
            .console()
            .split::<MmapFile, LogPacker>(
                "target/logs/temp.log",
                LogSize::MB(100),
                RollingType::All,
                LogPacker {},
            ),
    )
    .unwrap();
    b.iter(|| {
        black_box({
            log::info!("Commencing yak shaving");
        });
    });
}
