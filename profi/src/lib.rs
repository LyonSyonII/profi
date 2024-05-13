#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(clippy::needless_doctest_main)]

mod measure;
mod process;
pub mod zz_private;

/// Enables profiling for the annotated function.
///
/// Equivalent to putting [`prof!()`] at the start.
///
/// # Examples
/// ```rust
/// use profi::profile;
///
/// #[profile]
/// fn anotated() {
///     // ...
/// }
/// ```
#[cfg(feature = "attributes")]
pub use profi_attributes::profile;

/// Enables printing out the profiling results when `main` exits.
///
/// Equivalent to writing [`print_on_exit!()`] at the start of the function.
///
/// # Example
/// ```rust
/// #[profi::main]
/// fn main() {
///     // ...
/// }
/// ```
#[cfg(feature = "attributes")]
pub use profi_attributes::main;

pub use zz_private::Guard;

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

#[cfg(feature = "enable")]
pub(crate) type Str = beef::lean::Cow<'static, str>;

/// Profiles the time it takes for the scope to end.
///
/// If you want to get an explicit guard, use [`prof_guard!`].
///
/// # Examples
/// ## Infer function's name
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
///   sleep();
/// }
/// ```
///
/// ## Use provided name
/// ```
/// use profi::{prof, print_on_exit};
///
/// fn wait(ms: u64) {
///     // Profile `sleep`
///     prof!("wait millis");
///     std::thread::sleep(std::time::Duration::from_millis(ms));
/// }
///
/// fn main() {
///   print_on_exit!();
///   wait(15);
/// }
/// ```
///
/// ## Use dynamic name
/// ```
/// use profi::{prof, print_on_exit};
///
/// fn main() {
///   print_on_exit!();
///   
///   for i in 0..10 {
///     prof!(fmt = "iteration {i}");
///   }
/// }
/// ```
///
#[macro_export]
macro_rules! prof {
    ($($tt:tt)*) => {
        let _guard = $crate::prof_guard!($($tt)*);
    }
}

#[macro_export]
macro_rules! prof_block {
    ($($tt:tt)*) => {
        {
            let _guard = $crate::prof_guard!(stringify!($($tt)*));
            $($tt)*
        }
    };
}

/// Returns a guard that will profile as long as it's alive.
///
/// This will be until the scope ends or is dropped manually.
///
/// Supports the same syntax as [`prof!`];
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
            let name = $crate::zz_private::type_name_of(f);
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