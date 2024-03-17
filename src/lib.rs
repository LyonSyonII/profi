//! A simple, multithreaded profiler for Rust.
//!
//! Records the time it takes for a scope to end and print the timings to stdout or stderr when the program exits.
//!
//! Set the `enable` feature to use the profiler.  
//! If the feature is not enabled, all macros and methods will be no-ops and the timings will not be recorded.  
//! You can disable the feature adding `default-features = false` to the `miniprof` dependency in your `Cargo.toml`.
//!
//! # Examples
//!
//! Basic usage:
//! ```rust
//! use miniprof::{prof, print_on_exit};
//!
//! fn main() {
//!  // Prints the timings to stdout when the program exits
//!  // Always put at the top of the main function to ensure it's the last thing to run
//!  print_on_exit!();
//!
//!  // The `prof!` macro creates a guard that records the time until it goes out of scope
//!  prof!(main);
//!
//!  // Sleep for a bit to simulate some work
//!  std::thread::sleep(std::time::Duration::from_millis(200));
//! }
//! ```
//! ```plaintext
//! +------+-----------------+---------------+-------+
//! | Name | % of total time | Average time  | Calls |
//! +------+-----------------+---------------+-------+
//! | main | 100.00%         | 200.20ms/call |     1 |
//! +------+-----------------+---------------+-------+
//! ```
//!
//! Also supports loops or multiple calls to functions:
//! ```rust
//! use miniprof::{prof, print_timings};
//!
//! fn main() {
//!   for _ in 0..100 {
//!       prof!(loop);
//!       std::thread::sleep(std::time::Duration::from_millis(10));
//!   }
//!   print_timings().unwrap();
//! }
//! ```
//! ```plaintext
//! +------+-----------------+--------------+-------+
//! | Name | % of total time | Average time | Calls |
//! +------+-----------------+--------------+-------+
//! | loop | 100.00%         | 10.10ms/call |   100 |
//! +------+-----------------+--------------+-------+
//!```
//!
//! Works as expected when using multiple threads:
//! ```rust
//! use miniprof::{print_on_exit, prof};
//!
//! fn main() {
//!   print_on_exit!();
//!   
//!   std::thread::scope(|s| {
//!     // Spawn 10 threads
//!     for i in 0..10 {
//!       s.spawn(move || {
//!         for _ in 0..100 {
//!           // Need to bind it to a variable to ensure it doesn't go out of scope
//!           let _guard = if i < 6 {
//!               prof!("6 first threads")
//!           } else {
//!               prof!("4 last threads")
//!           };
//!           std::thread::sleep(std::time::Duration::from_millis(10));
//!           // The guard goes out of scope here
//!         }
//!       });
//!     }
//!   });
//! }
//! ```
//! ```plaintext
//! +-----------------+-----------------+--------------+-------+
//! | Name            | % of total time | Average time | Calls |
//! +-----------------+-----------------+--------------+-------+
//! | 6 first threads | 60.00%          | 10.08ms/call |   600 |
//! +-----------------+-----------------+--------------+-------+
//! | 4 last threads  | 40.00%          | 10.08ms/call |   400 |
//! +-----------------+-----------------+--------------+-------+
//! ```
#![allow(clippy::needless_doctest_main)]

use std::{cell::RefCell, rc::Rc};

use cli_table::WithTitle;

#[cfg(feature = "enable")]
#[derive(cli_table::Table, Debug, Clone)]
struct Timing {
    #[table(title = "Name")]
    name: String,
    #[table(title = "% Real Time", display_fn = "display_percent")]
    percent_real: f64,
    #[table(title = "Real Time", display_fn = "display_total")]
    total_real: std::time::Duration,
    #[table(title = "% CPU Time", display_fn = "display_percent")]
    percent_cpu: f64,
    #[table(title = "CPU Time", display_fn = "display_total")]
    total_cpu: std::time::Duration,
    #[table(
        title = "Average time",
        justify = "cli_table::format::Justify::Right",
        display_fn = "display_calls"
    )]
    average: std::time::Duration,
    #[table(title = "Calls", justify = "cli_table::format::Justify::Right")]
    calls: usize,
}

impl Timing {
    fn from_durations(
        name: &str,
        timings: &[std::time::Duration],
        total: std::time::Duration,
    ) -> Self {
        let sum = timings.iter().sum::<std::time::Duration>();
        let percent = (sum.as_secs_f64() / total.as_secs_f64()) * 100.0;
        let average = sum / timings.len() as u32;
        Self {
            name: name.to_string(),
            percent_real: percent,
            total_real: sum,
            percent_cpu: percent,
            total_cpu: sum,
            average,
            calls: timings.len(),
        }
    }
    fn merge(&mut self, other: &Timing) {
        self.average = (self.average + other.average) / 2;
        self.calls += other.calls;
        self.total_cpu += other.total_cpu;
    }
    fn update_percent(&mut self, total_real: std::time::Duration, total_cpu: std::time::Duration) {
        self.percent_real = (self.total_real.as_secs_f64() / total_real.as_secs_f64()) * 100.;
        self.percent_cpu = (self.total_cpu.as_secs_f64() / total_cpu.as_secs_f64()) * 100.;
    }
}

#[derive(Debug)]
struct GlobalProfiler {
    threads: std::sync::Mutex<usize>,
    cvar: std::sync::Condvar,
    timings: std::sync::RwLock<Vec<Vec<Timing>>>,
}

#[derive(Debug)]
struct ThreadProfiler {
    scopes: Vec<Rc<RefCell<ScopeProfiler>>>,
    current: Option<Rc<RefCell<ScopeProfiler>>>,
}

#[derive(Debug)]
struct ScopeProfiler {
    name: &'static str,
    timings: Vec<std::time::Duration>,
    parent: Option<Rc<RefCell<ScopeProfiler>>>,
    children: Vec<Rc<RefCell<ScopeProfiler>>>,
}

#[derive(Debug)]
pub struct ScopeGuard {
    instant: std::time::Instant,
}

static GLOBAL_PROFILER: GlobalProfiler = GlobalProfiler::new();

thread_local! {
    static THREAD_PROFILER: RefCell<ThreadProfiler> = RefCell::new(ThreadProfiler::new());
}

impl GlobalProfiler {
    const fn new() -> Self {
        Self {
            timings: std::sync::RwLock::new(Vec::new()),
            threads: std::sync::Mutex::new(0),
            cvar: std::sync::Condvar::new()
        }
    }

    fn print_timings(&self, to: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut local_timing = THREAD_PROFILER.with_borrow(|thread| thread.to_timings());
        let timings = &self.timings.read().unwrap();
        for thread in timings.iter() {
            for timing in thread {
                if let Some(t) = local_timing.iter_mut().find(|t| t.name == timing.name) {
                    t.merge(timing);
                } else {
                    local_timing.push(timing.clone());
                }
            }
        }
        let (total_real, total_cpu) = local_timing
            .iter()
            .map(|t| (t.total_real, t.total_cpu))
            .fold(
                (Default::default(), Default::default()),
                |(acc_r, acc_c): (std::time::Duration, _), (r, c)| (acc_r.max(r), acc_c + c),
            );

        local_timing
            .iter_mut()
            .for_each(|t| t.update_percent(total_real, total_cpu));
        write!(to, "{}", local_timing.with_title().display()?)
    }
}

impl ThreadProfiler {
    fn new() -> Self {
        *GLOBAL_PROFILER.threads.lock().unwrap() += 1;
        Self {
            scopes: Vec::new(),
            current: None,
        }
    }

    fn scope_total(&self) -> std::time::Duration {
        self.scopes
            .iter()
            .map(|s| s.borrow().timings.iter().sum::<std::time::Duration>())
            .sum()
    }

    fn total(&self) -> std::time::Duration {
        self.scopes.iter().map(|s| s.borrow().total()).sum()
    }

    fn to_timings(&self) -> Vec<Timing> {
        let total = self.total();
        self.scopes
            .iter()
            .flat_map(|scope| scope.borrow().to_timings(total))
            .collect()
    }

    fn push(&mut self, name: &'static str) {
        let node = if let Some(current) = &self.current {
            let mut current_mut = current.borrow_mut();
            if let Some(scope) = current_mut
                .children
                .iter()
                .find(|s| s.borrow().name == name)
            {
                scope.clone()
            } else {
                // Create new scope with 'current' as parent
                let scope = ScopeProfiler::new(name, Some(current.clone()));
                // Update current scope's children
                current_mut.children.push(scope.clone());
                scope
            }
        } else if let Some(scope) = self.scopes.iter().find(|s| RefCell::borrow(s).name == name) {
            scope.clone()
        } else {
            let scope = ScopeProfiler::new(name, None);
            self.scopes.push(scope.clone());
            scope
        };
        // Update current to new scope
        self.current = Some(node);
    }

    fn pop(&mut self, duration: std::time::Duration) {
        let Some(current) = &self.current else {
            panic!("[miniprof] 'pop' called and 'current' is 'None', this should never happen!")
        };
        RefCell::borrow_mut(current).timings.push(duration);
        let parent = current.borrow().parent.clone();
        self.current = parent;
    }
}

impl ScopeProfiler {
    fn new(name: &'static str, parent: Option<Rc<RefCell<ScopeProfiler>>>) -> Rc<RefCell<Self>> {
        let s = Self {
            name,
            parent,
            timings: Vec::new(),
            children: Vec::new(),
        };
        Rc::new(RefCell::new(s))
    }
    fn to_timings(&self, total: std::time::Duration) -> Vec<Timing> {
        let timing = Timing::from_durations(self.name, &self.timings, total);
        std::iter::once(timing)
            .chain(
                self.children
                    .iter()
                    .flat_map(|child| child.borrow().to_timings(total)),
            )
            .collect()
    }
    fn total(&self) -> std::time::Duration {
        self.timings.iter().sum::<std::time::Duration>()
            + self.children.iter().map(|c| c.borrow().total()).sum()
    }
}

impl ScopeGuard {
    pub fn new(name: &'static str) -> Self {
        THREAD_PROFILER.with_borrow_mut(|thread| thread.push(name));
        Self {
            instant: std::time::Instant::now(),
        }
    }
}

impl Drop for ThreadProfiler {
    fn drop(&mut self) {
        if !self.scopes.is_empty() {
            let timings = self.to_timings();
            GLOBAL_PROFILER.timings.write().unwrap().push(timings);
        }
        *GLOBAL_PROFILER.threads.lock().unwrap() -= 1;
        GLOBAL_PROFILER.cvar.notify_one()
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        THREAD_PROFILER.with_borrow_mut(|thread| {
            thread.pop(self.instant.elapsed());
        })
    }
}

/// Blocks until all threads are dropped.
/// 
/// Must be used on [`print_on_exit!`] because sometimes the threads will drop *after* the main one, corrupting the results.
/// 
/// Will be applied automatically on `print_on_exit!`, should not be used on its own.
#[inline(always)]
pub fn block_until_exited() {
    // Wait for all threads to finish
    let mut threads = GLOBAL_PROFILER.threads.lock().unwrap();
    while *threads > 1 {
        threads = GLOBAL_PROFILER.cvar.wait(threads).unwrap();
    }
}

/// Prints the profiled timings to stdout.
///
/// If profiling the `main` function, you can use [`print_on_exit!()`] instead.
#[inline(always)]
pub fn print_timings() -> std::io::Result<()> {
    #[cfg(feature = "enable")]
    GLOBAL_PROFILER.print_timings(&mut std::io::stdout().lock())?;
    Ok(())
}
/// Prints the profiled timings to stderr.
///
/// If profiling the `main` function, you can use [`print_on_exit!()`] instead.
#[inline(always)]
pub fn eprint_timings() -> std::io::Result<()> {
    #[cfg(feature = "enable")]
    GLOBAL_PROFILER.print_timings(&mut std::io::stderr())?;
    Ok(())
}
/// Prints the profiled timings to the provided [`std::io::Write`].
///
/// If profiling the `main` function, you can use [`print_on_exit!()`] instead.
#[inline(always)]
pub fn print_timings_to(to: &mut impl std::io::Write) -> std::io::Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(500));
    #[cfg(feature = "enable")]
    GLOBAL_PROFILER.print_timings(to)?;
    Ok(())
}

#[cfg(feature = "enable")]
#[inline(always)]
fn display_percent(f: &f64) -> String {
    format!("{:.2}%", f)
}

#[cfg(feature = "enable")]
#[inline(always)]
fn display_calls(d: &std::time::Duration) -> String {
    format!("{:.2?}/call", d)
}

#[cfg(feature = "enable")]
#[inline(always)]
fn display_total(d: &std::time::Duration) -> String {
    format!("{:.2?}", d)
}

/// Profiles the time it takes for the scope to end.
///
/// You can also bind it to a variable to stop the profiling early.
///
/// # Examples
/// ```
/// use miniprof::{prof, print_on_exit};
///
/// fn main() {
///   print_on_exit!();
///   prof!(main);
///
///   std::thread::sleep(std::time::Duration::from_millis(100));
///
///   // Bind it to a variable to stop the profiling early
///   let _guard = prof!("sleep2");
///   std::thread::sleep(std::time::Duration::from_millis(100));
///   _guard.stop();
/// }
/// ```
#[macro_export]
macro_rules! prof {
    ($name:ident) => {
        #[cfg(feature = "enable")]
        let _guard = $crate::ScopeGuard::new(stringify!($name));
    };
    ($name:literal) => {
        $crate::ScopeGuard::new($name)
    };
}

/// Prints the profiled timings to stdout when the `main` exits.
///
/// **Always put at the top of `main` to ensure it's dropped last.**
///
/// If you want to print to stderr instead, use `print_on_exit!(stderr)`.
///
/// # Examples
/// ```
/// use miniprof::{prof, print_on_exit};
///
/// fn main() {
///   print_on_exit!();
///   prof!(main);
///   std::thread::sleep(std::time::Duration::from_millis(200));
/// }
/// ```
///
/// Print to stderr instead of stdout:
/// ```
/// use miniprof::{prof, print_on_exit};
///
/// fn main() {
///   print_on_exit!(stderr);
///   prof!(main);
///   std::thread::sleep(std::time::Duration::from_millis(200));
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
#[macro_export]
macro_rules! print_on_exit {
    () => {
        print_on_exit!(stdout)
    };
    (stdout) => {
        print_on_exit!(to = || std::io::stdout())
    };
    (stderr) => {
        print_on_exit!(to = || std::io::stderr())
    };
    (to = $to:expr) => {
        #[cfg(feature = "enable")]
        struct MiniprofDrop<W: std::io::Write, F: Fn() -> W>(F);
        #[cfg(feature = "enable")]
        impl<W: std::io::Write, F: Fn() -> W> std::ops::Drop for MiniprofDrop<W, F> {
            fn drop(&mut self) {
                $crate::block_until_exited();
                $crate::print_timings_to(&mut (self.0)()).unwrap();
            }
        }
        #[cfg(feature = "enable")]
        let _guard = MiniprofDrop($to);
    };
}
