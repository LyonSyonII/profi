#![doc = include_str!("../../README.md")]
#![allow(clippy::needless_doctest_main)]

mod measure;
mod process;
pub mod zz_private;

#[cfg(feature = "attributes")]
pub use profi_attributes::profile;

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
