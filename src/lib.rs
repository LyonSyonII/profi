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

#[cfg(feature = "enable")]
struct GlobalProfiler {
    timings: once_cell::sync::Lazy<dashmap::DashMap<&'static str, Vec<std::time::Duration>>>,
}

#[cfg(feature = "enable")]
static GLOBAL_PROFILER: GlobalProfiler = GlobalProfiler {
    timings: once_cell::sync::Lazy::new(dashmap::DashMap::new),
};

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
    #[table(
        title = "Average time",
        justify = "cli_table::format::Justify::Right",
        display_fn = "display_duration"
    )]
    average: std::time::Duration,
    #[table(title = "Calls", justify = "cli_table::format::Justify::Right")]
    calls: usize,
}

#[cfg(feature = "enable")]
impl GlobalProfiler {
    fn print_timings(&self, to: &mut impl std::io::Write) -> std::io::Result<()> {
        use cli_table::WithTitle;

        let timings = &self.timings;
        let total: std::time::Duration = timings
            .iter()
            .map(|r| r.value().iter().sum::<std::time::Duration>())
            .sum();
        let mut timings = self
            .timings
            .iter()
            .map(|r| {
                let (name, timings) = r.pair();
                let sum = timings.iter().sum::<std::time::Duration>();
                let percent = (sum.as_secs_f64() / total.as_secs_f64()) * 100.0;
                let average = sum / timings.len() as u32;
                Timing {
                    name: name.to_string(),
                    percent,
                    average,
                    calls: timings.len(),
                }
            })
            .collect::<Vec<_>>();
        timings.sort_unstable_by(|a, b| b.percent.partial_cmp(&a.percent).unwrap());

        write!(to, "{}", timings.with_title().display()?)
    }
}

/// A guard that records the time from its instantiation to being dropped.
/// 
/// Should not be used directly, use the [`prof!`] macro instead.
pub struct LocalProfilerGuard {
    #[cfg(feature = "enable")]
    name: &'static str,
    #[cfg(feature = "enable")]
    start: std::time::Instant,
}

impl LocalProfilerGuard {
    /// Creates a new `LocalProfilerGuard` with the given name.
    /// 
    /// Should not be used directly, use the [`prof!`] macro instead.
    #[inline(always)]
    pub fn new(name: &'static str) -> Self {
        Self {
            #[cfg(feature = "enable")]
            name,
            #[cfg(feature = "enable")]
            start: std::time::Instant::now(),
        }
    }
    /// Stops the profiling early.
    ///
    /// Useful to avoid profiling the entire scope.
    ///
    /// # Examples
    /// ```
    /// use miniprof::{prof, print_on_exit};
    ///
    /// fn main() {
    ///   let _guard = prof!("sleep2");
    ///   std::thread::sleep(std::time::Duration::from_millis(100));
    ///   _guard.stop();
    /// }
    /// ```
    #[allow(clippy::needless_doctest_main)]
    pub fn stop(self) {
        #[cfg(feature = "enable")]
        {
            GLOBAL_PROFILER
                .timings
                .entry(self.name)
                .or_default()
                .push(self.start.elapsed());
            std::mem::forget(self);
        }
    }
}

#[cfg(feature = "enable")]
impl Drop for LocalProfilerGuard {
    fn drop(&mut self) {
        GLOBAL_PROFILER
            .timings
            .entry(self.name)
            .or_default()
            .push(self.start.elapsed());
    }
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
        let _guard = $crate::LocalProfilerGuard::new(stringify!($name));
    };
    ($name:literal) => {
        $crate::LocalProfilerGuard::new($name)
    };
}

/// Prints the profiled timings to stdout when the function exits.
///
/// **Always put at the top of the function to ensure it's dropped last.**
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
                $crate::print_timings_to(&mut (self.0)()).unwrap();
            }
        }
        #[cfg(feature = "enable")]
        let _guard = MiniprofDrop($to);
    };
}