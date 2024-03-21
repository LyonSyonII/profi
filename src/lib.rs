//! A simple profiler for single and multithreaded applications.
//!
//! Record the time it takes for a scope to end and print the timings when the program exits.
//!
//! Each measurement has an overhead of ~16-29ns, so it shouldn't impact benchmarks.  
//! Run the [benchmarks](https://github.com/LyonSyonII/miniprof/blob/main/examples/benchmark.rs) example to see what's the overhead on your machine.
//!
//! # Setup
//!
//! `miniprof` is controlled by the `enable` feature, which is active by default.  
//! When disabled, all macros and methods will become no-ops, resulting in zero impact on your code.
//!
//! To disable it, add `default-features = false` to the `miniprof` dependency in your `Cargo.toml`.
//!
//! For convenience, you can also add a custom feature:
//! ```toml
//! [dependencies]
//! miniprof = { version = "*", default-features = false }
//!
//! [features]
//! prof = ["miniprof/enable"]
//! ```
//!
//! And run it with `cargo run --release --features prof`
//!
//! # Usage
//!
//! ## Basic Usage
//! ```rust
//! use miniprof::{prof, print_on_exit};
//!
//! fn main() {
//!  // Prints the timings to stdout when the program exits
//!  // Always put at the top of the main function to ensure it's dropped last
//!  //
//!  // An implicit `main` guard is created to profile the whole application
//!  print_on_exit!();
//!
//!  // Sleep for a bit to simulate some work
//!  std::thread::sleep(std::time::Duration::from_millis(200));
//! }
//! ```
//! ```plaintext
//! ┌──────────────┬────────────────────┬───────────┬────────────┬──────────┬──────────────┬───────┐
//! │ Name         ┆ % Application Time ┆ Real Time ┆ % CPU Time ┆ CPU Time ┆ Average time ┆ Calls │
//! ╞══════════════╪════════════════════╪═══════════╪════════════╪══════════╪══════════════╪═══════╡
//! │ simple::main ┆ 100.00%            ┆ 200.13ms  ┆      -     ┆     -    ┆       -      ┆     1 │
//! └──────────────┴────────────────────┴───────────┴────────────┴──────────┴──────────────┴───────┘
//! ```
//!
//! ## Loops
//! ```rust
//! use miniprof::{prof, print_on_exit};
//!
//! fn main() {
//!   print_on_exit!();
//!
//!   for _ in 0..100 {
//!       prof!(iteration);
//!       std::thread::sleep(std::time::Duration::from_millis(10));
//!   }
//! }
//! ```
//! ```plaintext
//! ┌────────────┬────────────────────┬───────────┬────────────┬──────────┬──────────────┬───────┐
//! │ Name       ┆ % Application Time ┆ Real Time ┆ % CPU Time ┆ CPU Time ┆ Average time ┆ Calls │
//! ╞════════════╪════════════════════╪═══════════╪════════════╪══════════╪══════════════╪═══════╡
//! │ loop::main ┆ 100.00%            ┆ 1.01s     ┆      -     ┆     -    ┆       -      ┆     1 │
//! ├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
//! │ iteration  ┆ 99.99%             ┆ 1.01s     ┆      -     ┆     -    ┆ 10.10ms/call ┆   100 │
//! └────────────┴────────────────────┴───────────┴────────────┴──────────┴──────────────┴───────┘
//!```
//!
//! ## Multiple threads
//! ```rust
//! use miniprof::{print_on_exit, prof_guard};
//!
//! fn do_work(i: usize) {
//!     for _ in 0..100 {
//!         // Need to bind it to a variable to ensure it isn't dropped before sleeping
//!         let _guard = if i < 6 {
//!             prof_guard!("6 first")
//!         } else {
//!             prof_guard!("4 last")
//!         };
//!         std::thread::sleep(std::time::Duration::from_millis(10));
//!         // The guard goes out of scope here
//!     }
//! }
//!
//! fn main() {
//!     print_on_exit!();
//!
//!     std::thread::scope(|s| {
//!         for i in 0..10 {
//!             s.spawn(move || {
//!                 do_work(i);
//!             });
//!         }
//!     });
//! }
//! ```
//! ```plaintext
//! ┌───────────┬────────────────────┬───────────┬────────────┬──────────┬──────────────┬───────┐
//! │ Name      ┆ % Application Time ┆ Real Time ┆ % CPU Time ┆ CPU Time ┆ Average time ┆ Calls │
//! ╞═══════════╪════════════════════╪═══════════╪════════════╪══════════╪══════════════╪═══════╡
//! │ main      ┆ 100.00%            ┆ 1.01s     ┆      -     ┆     -    ┆       -      ┆     1 │
//! ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
//! │ 6 first   ┆ 99.98%             ┆ 1.01s     ┆ 54.55%     ┆ 6.04s    ┆ 10.08ms/call ┆   600 │
//! ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
//! │ 4 last    ┆ 99.96%             ┆ 1.01s     ┆ 36.36%     ┆ 4.03s    ┆ 10.07ms/call ┆   400 │
//! └───────────┴────────────────────┴───────────┴────────────┴──────────┴──────────────┴───────┘
//! ```
//! "CPU Time" is the combined time all threads have spent on that scope.  
//!
//! For example, "6 first" has a "CPU Time" of 6s because each thread waits 1s, and the program spawns six of them.
//!
//! # Features
//! - `enable`: Activates the profiling, if not active all macros become no-ops.
//! - `hierarchy`: Shows when a `prof!` is the child of another with a visual hierarchy. Can be expensive, disable it if you're measuring very precisely.
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
fn create_table(timings: Vec<Timing>) -> comfy_table::Table {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header([
        "Name",
        "% Application Time",
        "Real Time",
        "% CPU Time",
        "CPU Time",
        "Average time",
        "Calls",
    ]);

    let empty = || comfy_table::Cell::new("-").set_alignment(comfy_table::CellAlignment::Center);

    for timing in timings {
        fn cell(c: impl Into<comfy_table::Cell>) -> comfy_table::Cell {
            c.into()
        }

        let name = cell(timing.name);
        let app_percent = cell(format!("{:.2}%", timing.percent_app));
        let real_time = cell(format!("{:.2?}", timing.total_real));
        let (cpu_percent, cpu_time) = if timing.total_real == timing.total_cpu {
            (empty(), empty())
        } else {
            (
                cell(format!("{:.2}%", timing.percent_cpu)),
                cell(format!("{:.2?}", timing.total_cpu)),
            )
        };
        let average = if timing.average.is_zero() || timing.calls <= 1 {
            empty()
        } else {
            cell(format!("{:.2?}/call", timing.average))
        };
        let calls = if timing.calls == 0 {
            empty()
        } else {
            cell(timing.calls).set_alignment(comfy_table::CellAlignment::Right)
        };
        table.add_row([
            name,
            app_percent,
            real_time,
            cpu_percent,
            cpu_time,
            average,
            calls,
        ]);
    }

    table
}

#[cfg(feature = "enable")]
impl Timing {
    fn from_durations(
        name: String,
        timings: &[std::time::Duration],
        total: std::time::Duration,
    ) -> Self {
        let sum = timings.iter().sum::<std::time::Duration>();
        let percent = if !total.is_zero() {
            (sum.as_secs_f64() / total.as_secs_f64()) * 100.0
        } else {
            100.0
        };
        let average = sum / timings.len().max(1) as u32;
        Self {
            name,
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
    name: std::borrow::Cow<'static, str>,
    timings: Vec<std::time::Duration>,
    parent: std::rc::Weak<RefCell<ScopeProfiler>>,
    children: Vec<Rc<RefCell<ScopeProfiler>>>,
    hierarchy_depth: usize,
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

    fn print_timings(&self, mut to: impl std::io::Write) -> std::io::Result<()> {
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
        write!(to, "{}", create_table(local_timing))
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

    fn to_timings(&self) -> Vec<Timing> {
        let total = self.get_thread_time();
        let timings = self
            .scopes
            .iter()
            .flat_map(|scope| scope.borrow().to_timings(total))
            .collect();
        timings
    }

    fn push(&mut self, name: impl Into<std::borrow::Cow<'static, str>>) {
        let name = name.into();

        let node = if let Some(current) = &self.current {
            let mut current_mut = current.borrow_mut();
            // If 'name' is a child of 'current'
            if let Some(scope) = current_mut
                .children
                .iter()
                .find(|s| s.borrow().name == name)
            {
                scope.clone()
            }
            // If not, create new scope with 'current' as parent
            else {
                // Add visual indicator of the nesting
                let scope = ScopeProfiler::new(
                    name,
                    Rc::downgrade(current),
                    current_mut.hierarchy_depth + 1,
                );

                // Update current scope's children
                current_mut.children.push(scope.clone());
                scope
            }
        }
        // If 'name' is a root scope
        else if let Some(scope) = self.scopes.iter().find(|s| RefCell::borrow(s).name == name) {
            scope.clone()
        }
        // Else create a new root scope
        else {
            let scope = ScopeProfiler::new(name, std::rc::Weak::new(), 0);
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
        self.current = parent.upgrade();
    }
}

#[cfg(feature = "enable")]
impl ScopeProfiler {
    fn new(
        name: std::borrow::Cow<'static, str>,
        parent: std::rc::Weak<RefCell<ScopeProfiler>>,
        hierarchy_depth: usize,
    ) -> Rc<RefCell<Self>> {
        let s = Self {
            name,
            parent,
            hierarchy_depth,
            timings: Vec::new(),
            children: Vec::new(),
        };
        Rc::new(RefCell::new(s))
    }
    fn to_timings(&self, total: std::time::Duration) -> Vec<Timing> {
        #[cfg(feature = "hierarchy")]
        let name = {
            // Add a padding equal to hierarchy depth
            // If it's >= 20, add a numeric indicator and limit the padding
            let spaces = if self.hierarchy_depth >= 20 {
                let new = format!("(+{}) ", self.hierarchy_depth);
                format!("{}{new}", " ".repeat(20usize.saturating_sub(new.len())))
            } else {
                " ".repeat(self.hierarchy_depth)
            };
            format!("{}{}", spaces, self.name) 
        };
        #[cfg(not(feature = "hierarchy"))]
        let name = self.name.to_string();
        let timing = Timing::from_durations(name, &self.timings, total);
        std::iter::once(timing)
            .chain(
                self.children
                    .iter()
                    .flat_map(|child| child.borrow().to_timings(total)),
            )
            .collect()
    }
}

#[cfg(feature = "enable")]
impl Drop for ThreadProfiler {
    fn drop(&mut self) {
        #[cfg(not(feature = "rayon"))]
        self.manual_drop()
    }
}

/// Profiles the time it takes for the scope to end.
///
/// If you want to get an explicit guard, use [`prof_guard!`].
///
/// # Examples
/// ```
/// use miniprof::{prof, print_on_exit};
///
/// fn sleep() {
///     // Profile `sleep`
///     prof!();
///     std::thread::sleep(std::time::Duration::from_millis(200));
/// }
///
/// fn main() {
///   print_on_exit!();
///
///   sleep();
/// }
/// ```
#[macro_export]
macro_rules! prof {
    ($($tt:tt)*) => {
        let _guard = $crate::prof_guard!($($tt)*);
    }
}

/// Returns a guard that will profile as long as it's alive.
///
/// This will be until the scope ends or dropped manually.
///
/// # Examples
/// ```
/// use miniprof::{prof_guard, print_on_exit};
///
/// fn sleep(time: u64) {
///   // Must be saved into an explicit guard, or it will be dropped at the end of the `if` block
///   let _guard = if time < 100 {
///     prof_guard!("< 100")
///   } else {
///     prof_guard!(">= 100")
///   };
///   std::thread::sleep(std::time::Duration::from_millis(time));
/// }
///
/// fn main() {
///   print_on_exit!();
///
///   sleep(50);
///   sleep(150);
/// }
/// ```
#[macro_export]
macro_rules! prof_guard {
    () => {
        $crate::prof_guard!({
            // https://docs.rs/stdext/latest/src/stdext/macros.rs.html#63-74
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            // `3` is the length of the `::f`.
            &name[..name.len() - 3]
        })
    };
    ($name:ident) => {
        $crate::prof_guard!(stringify!($name))
    };
    (fmt = $( $name:tt )+) => {
        $crate::prof_guard!(format!($($name)+))
    };
    ($name:expr) => {
        $crate::zz_private::ScopeGuard::new($name)
    };
}

/// Prints the profiled timings to stdout when `main` exits.
///
/// Creates an implicit `main` profiling guard, which will profile the whole program's time.
///
/// **Always put at the top of the `main` function to ensure it's dropped last.**
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
///   // ...
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
///   print_on_exit!(to = &mut file, ondrop = |f| println!("{f:?}"));
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
        $crate::print_on_exit!(to = $to, ondrop = |_| {})
    };
    (to = $to:expr, ondrop = $ondrop:expr) => {
        let mut _to = $to;
        let _guard = $crate::zz_private::MiniprofDrop::new(&mut _to, $ondrop);
        // Implicit guard for profiling the whole application
        $crate::prof!()
    };
}
