#[derive(Debug)]
pub struct Counter {
    total: usize,
    current: usize,
}

impl Counter {
    pub fn start(total: usize) -> Counter {
        log::info!("start - {} tasks to execute", total);
        Counter { total, current: 0 }
    }
    pub fn increase(&mut self) {
        self.current += 1;
        if self.current % 100_usize == 0 || self.current == self.total {
            log::info!(
                "progress - {}% - {}/{} tasks executed",
                self.current * 100 / self.total,
                self.current,
                self.total,
            )
        }
    }
    pub fn stop(&self) {
        log::info!("stop - {} tasks have been executed", self.total)
    }
}
