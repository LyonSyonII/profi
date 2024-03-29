#[allow(unused)]
use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "enable")]
use crate::Str;

#[cfg(feature = "enable")]
#[derive(Debug, Clone)]
pub(crate) struct Measure {
    pub(crate) ty: MeasureType,
    pub(crate) time: minstant::Instant,
}

#[cfg(feature = "enable")]
#[derive(Debug, Clone)]
pub(crate) enum MeasureType {
    Start { name: Str },
    End,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
pub(crate) struct GlobalProfiler {
    pub(crate) threads: std::sync::Mutex<usize>,
    pub(crate) cvar: std::sync::Condvar,
    measures: std::sync::RwLock<Vec<(std::time::Duration, Vec<Measure>)>>,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
pub(crate) struct ThreadProfiler {
    measures: Vec<Measure>,
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
            measures: std::sync::RwLock::new(Vec::new()),
            threads: std::sync::Mutex::new(0),
            cvar: std::sync::Condvar::new(),
        }
    }

    pub(crate) fn print_timings(&self, to: impl std::io::Write) -> std::io::Result<()> {
        THREAD_PROFILER.with_borrow(|thread| {
            thread.get_thread_time();
        });

        crate::process::print_timings(self.measures.read().unwrap().as_slice(), to)
    }
}

#[cfg(feature = "enable")]
impl ThreadProfiler {
    pub(crate) fn new() -> Self {
        *GLOBAL_PROFILER.threads.lock().unwrap() += 1;
        Self {
            measures: Vec::with_capacity(4096),
            thread_start: minstant::Instant::now(),
            thread_time: None,
        }
    }

    pub(crate) fn push(&mut self, name: Str, time: minstant::Instant) {
        self.measures.push(Measure {
            time,
            ty: MeasureType::Start { name },
        });
    }

    pub(crate) fn pop(&mut self, time: minstant::Instant) {
        self.measures.push(Measure {
            time,
            ty: MeasureType::End,
        })
    }

    pub(crate) fn manual_drop(&mut self, main_thread: bool) {
        self.set_thread_time();
        let measures = std::mem::take(&mut self.measures);
        if !measures.is_empty() {
            let mut lock = GLOBAL_PROFILER.measures.write().unwrap();
            if main_thread {
                // Ensure the main thread is always first
                lock.insert(0, (self.thread_time.unwrap(), measures));
            } else {
                lock.push((self.thread_time.unwrap(), measures));
            }
        }
        if !main_thread {
            let mut lock = GLOBAL_PROFILER.threads.lock().unwrap();
            *lock -= 1;
            GLOBAL_PROFILER.cvar.notify_one()
        }
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
        self.manual_drop(false)
    }
}
