#[allow(unused)]
use std::{cell::RefCell, rc::Rc};

use crate::Str;

#[derive(Debug, Clone)]
struct Measure {
    ty: MeasureType,
    time: minstant::Instant,
}

#[derive(Debug, Clone)]
enum MeasureType {
    Start { name: Str },
    End { id: usize },
}

#[cfg(feature = "enable")]
#[derive(Debug)]
pub(crate) struct GlobalProfiler {
    pub(crate) threads: std::sync::Mutex<usize>,
    pub(crate) cvar: std::sync::Condvar,
    measurements: std::sync::RwLock<Vec<(std::time::Duration, Vec<Measure>)>>,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
pub(crate) struct ThreadProfiler {
    measurements: Vec<Measure>,
    thread_start: minstant::Instant,
    thread_time: Option<std::time::Duration>,
}

#[cfg(feature = "enable")]
pub(crate) static GLOBAL_PROFILER: GlobalProfiler = GlobalProfiler::new();

#[cfg(feature = "enable")]
thread_local! {
    pub(crate) static THREAD_PROFILER: RefCell<ThreadProfiler> = RefCell::new(ThreadProfiler::new());
}

#[cfg(feature = "enable")]
impl GlobalProfiler {
    const fn new() -> Self {
        Self {
            measurements: std::sync::RwLock::new(Vec::new()),
            threads: std::sync::Mutex::new(0),
            cvar: std::sync::Condvar::new(),
        }
    }

    pub(crate) fn print_timings(&self, mut to: impl std::io::Write) -> std::io::Result<()> {
        dbg!(THREAD_PROFILER.with_borrow(|thread| thread.get_thread_time()));
        return Ok(());
        let mut measurements = THREAD_PROFILER.with_borrow(|thread| thread.measurements.clone());
        { 
            let m = self.measurements.read().unwrap();
            measurements.extend(m.iter().flat_map(|m| m.1.clone()));
        }
        write!(to, "measurements: {measurements:#?}")
    }
}

#[cfg(feature = "enable")]
impl ThreadProfiler {
    pub(crate) fn new() -> Self {
        *GLOBAL_PROFILER.threads.lock().unwrap() += 1;
        Self {
            measurements: Vec::new(),
            thread_start: minstant::Instant::now(),
            thread_time: None,
        }
    }

    pub(crate) fn push(&mut self, name: Str, time: minstant::Instant) -> usize {
        self.measurements.push(Measure {
            time,
            ty: MeasureType::Start { name },
        });
        self.measurements.len() - 1
    }

    pub(crate) fn pop(&mut self, id: usize, time: minstant::Instant) {
        self.measurements.push(Measure {
            time,
            ty: MeasureType::End { id },
        })
    }

    pub(crate) fn manual_drop(&mut self) {
        self.set_thread_time();
        if !self.measurements.is_empty() {
            GLOBAL_PROFILER
                .measurements
                .write()
                .unwrap()
                .push((self.thread_time.unwrap(), self.measurements.clone()));
        }
        *GLOBAL_PROFILER.threads.lock().unwrap() -= 1;
        GLOBAL_PROFILER.cvar.notify_one()
    }

    pub(crate) fn set_thread_time(&mut self) {
        self.thread_time.replace(self.thread_start.elapsed());
    }

    pub(crate) fn get_thread_time(&self) -> std::time::Duration {
        match self.thread_time {
            Some(t) => t,
            None => self.thread_start.elapsed(),
        }
    }
}

#[cfg(feature = "enable")]
impl Drop for ThreadProfiler {
    fn drop(&mut self) {
        #[cfg(not(feature = "rayon"))]
        self.manual_drop()
    }
}
