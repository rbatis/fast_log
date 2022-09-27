pub trait TPS {
    fn tps(&self, total: u64);
    fn time(&self, total: u64);
    fn cost(&self);
}

impl TPS for std::time::Instant {
    fn tps(&self, total: u64) {
        let time = self.elapsed();
        println!(
            "TPS: {} Iter/s",
            (total as u128 * 1000000000 as u128 / time.as_nanos() as u128)
        );
    }

    fn time(&self, total: u64) {
        let time = self.elapsed();
        println!(
            "Time: {:?} ,each:{} ns/op",
            &time,
            time.as_nanos() / (total as u128)
        );
    }

    fn cost(&self) {
        let time = self.elapsed();
        println!("cost:{:?}", time);
    }
}
