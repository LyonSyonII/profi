//! A simple profiler for single and multithreaded applications.
//!
//! Record the time it takes for a scope to end and print the timings when the program exits.
//!
//! Each measurement has an overhead of ~26ns, so it shouldn't impact benchmarks.  
//! Run the [benchmarks](https://github.com/LyonSyonII/profi/blob/main/examples/benchmark.rs) example to see what's the overhead on your machine.
//!
//! # Setup
//!
//! `profi` is controlled by the `enable` feature, which is active by default.  
//! When disabled, all macros and methods will become no-ops, resulting in zero impact on your code.
//!
//! To disable it, add `default-features = false` to the `profi` dependency in your `Cargo.toml`.
//!
//! For convenience, you can also add a custom feature:
//! ```toml
//! [dependencies]
//! profi = { version = "*", default-features = false }
//!
//! [features]
//! prof = ["profi/enable"]
//! ```
//!
//! And run it with `cargo run --release --features prof`
//!
//! # Usage
//!
//! ## Basic Usage
//! ```rust
//! use profi::{prof, print_on_exit};
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
//! use profi::{prof, print_on_exit};
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
//! use profi::{print_on_exit, prof_guard};
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
//! │  6 first  ┆ 99.98%             ┆ 1.01s     ┆ 54.55%     ┆ 6.04s    ┆ 10.08ms/call ┆   600 │
//! ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
//! │  4 last   ┆ 99.96%             ┆ 1.01s     ┆ 36.36%     ┆ 4.03s    ┆ 10.07ms/call ┆   400 │
//! └───────────┴────────────────────┴───────────┴────────────┴──────────┴──────────────┴───────┘
//! ```
//! "CPU Time" is the combined time all threads have spent on that scope.  
//!
//! For example, "6 first" has a "CPU Time" of 6s because each thread waits 1s, and the program spawns six of them.
//!
//! # Features
//! - `enable`: Activates the profiling, if not active all macros become no-ops.
#![allow(clippy::needless_doctest_main)]

mod measure;
mod process;
pub mod zz_private;

#[cfg(feature = "enable")]
pub(crate) type Str = beef::lean::Cow<'static, str>;

/// Allows profiling the profiling methods
#[allow(unused)]
macro_rules! meta_prof {
    ($name:ident) => {
        #[cfg(feature = "metaprof")]
        struct MetaProf {
            instant: minstant::Instant,
        }
        #[cfg(feature = "metaprof")]
        impl Drop for MetaProf {
            fn drop(&mut self) {
                let $name = self.instant.elapsed();
                dbg!($name);
            }
        }
        #[cfg(feature = "metaprof")]
        let _guard = MetaProf {
            instant: minstant::Instant::now(),
        };
    };
}

/// Profiles the time it takes for the scope to end.
///
/// If you want to get an explicit guard, use [`prof_guard!`].
///
/// # Examples
/// ```
/// use profi::{prof, print_on_exit};
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
/// use profi::{prof_guard, print_on_exit};
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
/// use profi::{prof, print_on_exit};
///
/// fn main() {
///   print_on_exit!();
///   // ...
/// }
/// ```
///
/// Print to stderr instead of stdout:
/// ```
/// use profi::{prof, print_on_exit};
///
/// fn main() {
///   print_on_exit!(stderr);
///   // ...
/// }
/// ```
///
/// Print to a file:
/// ```
/// use profi::{prof, print_on_exit};
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
        let _guard = $crate::zz_private::ProfiDrop::new(&mut _to, $ondrop);
        // Implicit guard for profiling the whole application
        $crate::prof!()
    };
}
