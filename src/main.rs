use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref as _;
use std::thread_local;

pub struct __GlobalProfiler {
    timings: once_cell::sync::Lazy<dashmap::DashMap<&'static str, Vec<std::time::Duration>>>
}

pub static GLOBAL_PROFILER: __GlobalProfiler = __GlobalProfiler {
    timings: once_cell::sync::Lazy::new(dashmap::DashMap::new)
};

impl __GlobalProfiler {
    pub fn print_timings(&self) {
        println!("Timings:");
        let timings = &self.timings;
        for r in timings.iter() {
            let (name, timings) = r.pair();
            let total: std::time::Duration = timings.iter().sum();
            let average = total / timings.len() as u32;
            println!("{}: total: {:?}, average: {:?}", name, total, average);
        }
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct __LocalProfiler {
    name: &'static str,
    timings: Vec<std::time::Duration>
}

impl __LocalProfiler {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            timings: Vec::new()
        }
    }
}

impl Drop for __LocalProfiler {
    fn drop(&mut self) {
        
    }
}

pub struct __LocalProfilerGuard {
    name: &'static str,
    start: std::time::Instant,
}

impl __LocalProfilerGuard {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for __LocalProfilerGuard {
    fn drop(&mut self) {
        GLOBAL_PROFILER.timings.entry(self.name).or_default().push(self.start.elapsed());
    }
}

#[macro_export]
macro_rules! prof {
    ($name:ident) => {
        let _guard = $crate::__LocalProfilerGuard::new(stringify!($name));
    };
}

fn main() {
    {
        prof!(main);
        let handle = std::thread::spawn(|| {
            for i in 0..10 {
                prof!(thread1);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
        for i in 0..100 {
            prof!(main_thread);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        handle.join().unwrap();
    }
    GLOBAL_PROFILER.print_timings();
}