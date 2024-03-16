#[cfg(not(feature = "enable"))]
struct GlobalProfiler;

#[cfg(feature = "enable")]
struct GlobalProfiler {
    timings: once_cell::sync::Lazy<dashmap::DashMap<&'static str, Vec<std::time::Duration>>>
}

#[cfg(not(feature = "enable"))]
static GLOBAL_PROFILER: GlobalProfiler = GlobalProfiler;

#[cfg(feature = "enable")]
static GLOBAL_PROFILER: GlobalProfiler = GlobalProfiler {
    timings: once_cell::sync::Lazy::new(dashmap::DashMap::new)
};

pub fn print_timings() -> std::io::Result<()> {
    GLOBAL_PROFILER.print_timings(&mut std::io::stdout().lock())
}

pub fn eprint_timings() -> std::io::Result<()> {
    GLOBAL_PROFILER.print_timings(&mut std::io::stderr())
}

pub fn print_timings_to(to: &mut impl std::io::Write) -> std::io::Result<()> {
    GLOBAL_PROFILER.print_timings(to)
}

#[cfg(feature = "enable")]
fn display_percent(f: &f64) -> String {
    format!("{:.2}%", f)
}

#[cfg(feature = "enable")]
fn display_duration(d: &std::time::Duration) -> String {
    format!("{:.2?}/call", d)
}

#[cfg(feature = "enable")]
#[derive(cli_table::Table)]
struct Timing {
    #[table(title = "Name")]
    name: String,
    #[table(title = "% of total time", display_fn = "display_percent")]
    percent: f64,
    #[table(title = "Average time", justify="cli_table::format::Justify::Right", display_fn = "display_duration")]
    average: std::time::Duration,
    #[table(title = "Calls", justify="cli_table::format::Justify::Right")]
    calls: usize
}

impl GlobalProfiler {
    #[cfg(feature = "enable")]
    fn print_timings(&self, to: &mut impl std::io::Write) -> std::io::Result<()> {
        use cli_table::WithTitle;

        let timings = &self.timings;
        let total: std::time::Duration = timings.iter().map(|r| r.value().iter().sum::<std::time::Duration>()).sum();
        let mut timings = self.timings.iter().map(|r| {
            let (name, timings) = r.pair();
            let sum = timings.iter().sum::<std::time::Duration>();
            let percent = (sum.as_secs_f64() / total.as_secs_f64()) * 100.0;
            let average = sum / timings.len() as u32;
            Timing {
                name: name.to_string(),
                percent,
                average,
                calls: timings.len()
            }
        }).collect::<Vec<_>>();
        timings.sort_unstable_by(|a, b| b.percent.partial_cmp(&a.percent).unwrap());
        
        write!(to, "{}", timings.with_title().display()?)
    }
    #[cfg(not(feature = "enable"))]
    #[inline(always)]
    pub fn print_timings(&self, to: &mut impl std::io::Write) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "enable")]
pub struct LocalProfilerGuard {
    name: &'static str,
    start: std::time::Instant,
}

#[cfg(not(feature = "enable"))]
pub struct __LocalProfilerGuard;

impl LocalProfilerGuard {
    #[cfg(feature = "enable")]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
    #[cfg(not(feature = "enable"))]
    #[inline(always)]
    pub fn new(name: &'static str) -> Self {
        Self
    }
    #[inline(always)]
    pub fn stop(self) {
        #[cfg(feature = "enable")] {
            GLOBAL_PROFILER.timings.entry(self.name).or_default().push(self.start.elapsed());
            std::mem::forget(self);
        }
    }
}

#[cfg(feature = "enable")]
impl Drop for LocalProfilerGuard {
    fn drop(&mut self) {
        GLOBAL_PROFILER.timings.entry(self.name).or_default().push(self.start.elapsed());
    }
}

#[cfg(feature = "enable")]
#[macro_export]
macro_rules! prof {
    ($name:ident) => {
        let _guard = $crate::LocalProfilerGuard::new(stringify!($name));
    };
    ($name:literal) => {
        $crate::LocalProfilerGuard::new($name)
    }
}

#[cfg(not(feature = "enable"))]
#[macro_export]
macro_rules! prof {
    ($name:ident) => {
        let _guard = $crate::LocalProfilerGuard;
    };
    ($name:literal) => {
        $crate::LocalProfilerGuard
    };
}

#[cfg(feature = "enable")]
#[macro_export]
macro_rules! print_on_exit {
    () => {
        struct Drop;
        impl std::ops::Drop for Drop {
            fn drop(&mut self) {
                $crate::print_timings().unwrap();
            }
        }
        let _guard = Drop;
    };
    (stdout) => {
        print_on_exit!();
    };
    (stderr) => {
        struct Drop;
        impl std::ops::Drop for Drop {
            fn drop(&mut self) {
                $crate::eprint_timings().unwrap();
            }
        }
        let _guard = Drop;
    };
}