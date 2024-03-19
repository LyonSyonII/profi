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
//! use miniprof::{prof, print_on_exit};
//!
//! fn main() {
//!   print_on_exit!();
//!
//!   for _ in 0..100 {
//!       prof!(loop);
//!       std::thread::sleep(std::time::Duration::from_millis(10));
//!   }
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
#![deny(unsafe_code)]

pub mod zz_private;

#[allow(unused)]
use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "enable")]
#[derive(Debug, Clone)]
struct Timing {
    name: String,
    /// % Application Time
    percent_app: f64,
    /// Real Time
    total_real: std::time::Duration,
    /// % CPU Time
    percent_cpu: f64,
    /// CPU Time
    total_cpu: std::time::Duration,
    average: std::time::Duration,
    calls: usize,
}

#[cfg(feature = "enable")]
impl ::cli_table::Title for Timing {
    fn title() -> ::cli_table::RowStruct {
        let title: ::std::vec::Vec<::cli_table::CellStruct> = ::std::vec![
            ::cli_table::Style::bold(::cli_table::Cell::cell("Name"), true),
            ::cli_table::Style::bold(::cli_table::Cell::cell("% Application Time"), true),
            ::cli_table::Style::bold(::cli_table::Cell::cell("Real Time"), true),
            ::cli_table::Style::bold(::cli_table::Cell::cell("% CPU Time"), true),
            ::cli_table::Style::bold(::cli_table::Cell::cell("CPU Time"), true),
            ::cli_table::Style::bold(::cli_table::Cell::cell("Average time"), true),
            ::cli_table::Style::bold(::cli_table::Cell::cell("Calls"), true),
        ];
        ::cli_table::Row::row(title)
    }
}

#[cfg(feature = "enable")]
impl ::cli_table::Row for &Timing {
    fn row(self) -> ::cli_table::RowStruct {
        use ::cli_table::format::Justify;
        let empty = || ::cli_table::Cell::cell("-").justify(Justify::Center);

        let mut row = vec![
            ::cli_table::Cell::cell(&self.name),
            ::cli_table::Cell::cell(display_percent(&self.percent_app)),
            ::cli_table::Cell::cell(display_total(&self.total_real)),
        ];

        if self.total_real != self.total_cpu {
            row.extend([
                ::cli_table::Cell::cell(display_percent(&self.percent_cpu)),
                ::cli_table::Cell::cell(display_total(&self.total_cpu)),
            ])
        } else {
            row.extend([empty(), empty()])
        }
        let average = if self.average.is_zero() || self.calls <= 1 {
            empty()
        } else {
            ::cli_table::Cell::cell(display_calls(&self.average))
                .justify(cli_table::format::Justify::Right)
        };
        let calls = if self.calls == 0 {
            empty()
        } else {
            ::cli_table::Cell::cell(self.calls).justify(cli_table::format::Justify::Right)
        };
        row.extend([average, calls]);
        ::cli_table::Row::row(row)
    }
}

#[cfg(feature = "enable")]
impl ::cli_table::Row for Timing {
    fn row(self) -> ::cli_table::RowStruct {
        #[allow(clippy::needless_borrows_for_generic_args)]
        ::cli_table::Row::row(&self)
    }
}

/* impl cli_table::Table for Timing {
    fn table(self) -> cli_table::TableStruct {

    }
} */

#[cfg(feature = "enable")]
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
            percent_app: percent,
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
        self.total_real = self.total_real.max(other.total_real);
    }
    fn update_percent(&mut self, total_app: std::time::Duration, total_cpu: std::time::Duration) {
        self.percent_app = (self.total_real.as_secs_f64() / total_app.as_secs_f64()) * 100.;
        self.percent_cpu = (self.total_cpu.as_secs_f64() / total_cpu.as_secs_f64()) * 100.;
    }
}

#[cfg(feature = "enable")]
#[derive(Debug)]
struct GlobalProfiler {
    threads: std::sync::Mutex<usize>,
    cvar: std::sync::Condvar,
    timings: std::sync::RwLock<Vec<(std::time::Duration, Vec<Timing>)>>,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
struct ThreadProfiler {
    scopes: Vec<Rc<RefCell<ScopeProfiler>>>,
    current: Option<Rc<RefCell<ScopeProfiler>>>,
    thread_start: std::time::Instant,
    thread_time: Option<std::time::Duration>,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
struct ScopeProfiler {
    name: &'static str,
    timings: Vec<std::time::Duration>,
    parent: Option<Rc<RefCell<ScopeProfiler>>>,
    children: Vec<Rc<RefCell<ScopeProfiler>>>,
}

#[cfg(feature = "enable")]
static GLOBAL_PROFILER: GlobalProfiler = GlobalProfiler::new();

#[cfg(feature = "enable")]
thread_local! {
    static THREAD_PROFILER: RefCell<ThreadProfiler> = RefCell::new(ThreadProfiler::new());
}

#[cfg(feature = "enable")]
impl GlobalProfiler {
    const fn new() -> Self {
        Self {
            timings: std::sync::RwLock::new(Vec::new()),
            threads: std::sync::Mutex::new(0),
            cvar: std::sync::Condvar::new(),
        }
    }

    fn print_timings(&self, to: &mut impl std::io::Write) -> std::io::Result<()> {
        let (mut total_app, mut local_timing) = THREAD_PROFILER.with_borrow(|thread| {
            let timings = thread.to_timings();
            (
                // Get first time if `print_on_exit!` has been used, or compute the thread time
                timings
                    .first()
                    .map(|t| t.total_real)
                    .unwrap_or_else(|| thread.get_thread_time()),
                timings,
            )
        });
        let timings = &self.timings.read().unwrap();
        for (thread_total, thread) in timings.iter() {
            total_app = std::cmp::max(total_app, *thread_total);
            for timing in thread {
                if let Some(t) = local_timing.iter_mut().find(|t| t.name == timing.name) {
                    t.merge(timing);
                } else {
                    local_timing.push(timing.clone());
                }
            }
        }
        let total_cpu = local_timing.iter().map(|t| t.total_cpu).sum();

        local_timing
            .iter_mut()
            .for_each(|t| t.update_percent(total_app, total_cpu));
        write!(
            to,
            "{}",
            cli_table::WithTitle::with_title(&local_timing).display()?
        )
    }
}

#[cfg(feature = "enable")]
impl ThreadProfiler {
    fn new() -> Self {
        *GLOBAL_PROFILER.threads.lock().unwrap() += 1;
        Self {
            scopes: Vec::new(),
            current: None,
            thread_start: std::time::Instant::now(),
            thread_time: None,
        }
    }

    fn manual_drop(&mut self) {
        self.set_thread_time();
        if !self.scopes.is_empty() {
            let timings = self.to_timings();
            GLOBAL_PROFILER
                .timings
                .write()
                .unwrap()
                .push((self.thread_time.unwrap(), timings));
        }
        *GLOBAL_PROFILER.threads.lock().unwrap() -= 1;
        GLOBAL_PROFILER.cvar.notify_one()
    }

    fn set_thread_time(&mut self) {
        self.thread_time.replace(self.thread_start.elapsed());
    }

    fn get_thread_time(&self) -> std::time::Duration {
        match self.thread_time {
            Some(t) => t,
            None => self.thread_start.elapsed(),
        }
    }

    fn total(&self) -> std::time::Duration {
        self.scopes.iter().map(|s| s.borrow().total()).sum()
    }

    fn to_timings(&self) -> Vec<Timing> {
        let total = self.get_thread_time();
        let timings = self
            .scopes
            .iter()
            .flat_map(|scope| scope.borrow().to_timings(total))
            .collect();
        timings
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

#[cfg(feature = "enable")]
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

#[cfg(feature = "enable")]
impl Drop for ThreadProfiler {
    fn drop(&mut self) {
        #[cfg(not(target = "rayon"))]
        self.manual_drop()
    }
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
///   drop(_guard);
/// }
/// ```
#[macro_export]
macro_rules! prof {
    () => {
        let _guard = $crate::zz_private::ScopeGuard::new({
            // https://docs.rs/stdext/latest/src/stdext/macros.rs.html#63-74
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            // `3` is the length of the `::f`.
            &name[..name.len() - 3]
        });
    };
    ($name:ident) => {
        let _guard = $crate::zz_private::ScopeGuard::new(stringify!($name));
    };
    ($name:literal) => {
        $crate::zz_private::ScopeGuard::new($name)
    };
}

/// Prints the profiled timings to stdout when `main` exits.
///
/// **Always put at the top of `main` to ensure it's dropped last.**
///
/// Print to stderr instead with `print_on_exit!(stderr)`.
///
/// Or print to a `std::io::Write` with `print_on_exit!(to = std::io::stdout())`
///
/// # Examples
/// ```
/// use miniprof::{prof, print_on_exit};
///
/// fn main() {
///   print_on_exit!();
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
///   // ...
/// }
/// ```
///
/// Print to a file:
/// ```
/// use miniprof::{prof, print_on_exit};
///
/// fn main() {
///   let mut file = Vec::<u8>::new();
///   print_on_exit!(to = &mut file);
///   // ...
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
#[macro_export]
macro_rules! print_on_exit {
    () => {
        $crate::print_on_exit!(stdout)
    };
    (stdout) => {
        $crate::print_on_exit!(to = std::io::stdout())
    };
    (stderr) => {
        $crate::print_on_exit!(to = std::io::stderr())
    };
    (to = $to:expr) => {
        // Wakes up the main thread's profiler to ensure that the `main` time is recorded properly
        let mut _to = $to;
        let _guard = $crate::zz_private::MiniprofDrop::new(&mut _to);
        // $crate::zz_private::init_profiler();
        prof!()
    };
}
