//! Contains all macro items, which should not be used by themselves.
//!
//! Always prefer the macros [`prof!`](crate::prof), [`prof_guard!`](crate::prof_guard) and [`print_on_exit!`](crate::print_on_exit).

#[cfg(feature = "enable")]
use crate::Str;
#[cfg(not(feature = "enable"))]
type Str = String;

#[cfg_attr(feature = "enable", derive(Debug))]
pub struct ScopeGuard {}

impl ScopeGuard {
    #[inline(always)]
    #[allow(unused)]
    pub fn new(name: impl Into<Str>) -> Self {
        #[cfg(feature = "enable")]
        crate::measure::THREAD_PROFILER.with_borrow_mut(|thread| thread.push(name.into()));
        Self {}
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        #[cfg(feature = "enable")]
        {
            // Do the measure as early as possible
            let time = minstant::Instant::now();
            crate::measure::THREAD_PROFILER.with_borrow_mut(|thread| {
                thread.pop(time);
            })
        }
    }
}

#[allow(dead_code)]
pub struct ProfiDrop<W: std::io::Write, F: Fn(&mut W)>(W, F);

impl<W, F> ProfiDrop<W, F>
where
    W: std::io::Write,
    F: Fn(&mut W),
{
    pub fn new(to: W, ondrop: F) -> Self {
        Self(to, ondrop)
    }
}

#[cfg(feature = "enable")]
impl<W, F> std::ops::Drop for ProfiDrop<W, F>
where
    W: std::io::Write,
    F: Fn(&mut W),
{
    fn drop(&mut self) {
        drop_threads();
        block_until_exited();
        print_timings_to(&mut self.0).unwrap();
        let s = &self.1;
        s(&mut self.0)
    }
}

#[inline(always)]
pub fn dbg_thread() {
    #[cfg(feature = "enable")]
    crate::measure::THREAD_PROFILER.with_borrow(|t| println!("{t:#?}"));
}

#[cfg(feature = "enable")]
fn drop_threads() {
    crate::measure::THREAD_PROFILER.with_borrow_mut(|t| {
        t.manual_drop(true);

        #[cfg(feature = "rayon")]
        {
            // Drop threads manually, as `rayon` never drops them
            let current = std::thread::current().id();

            rayon::broadcast(|t| {
                if std::thread::current().id() != current {
                    crate::measure::THREAD_PROFILER.with_borrow_mut(|t| t.manual_drop(false))
                }
            });
        }
    });
}

/// **Should not be used on its own, will be applied automatically with `print_on_exit!`.**
///
/// Blocks until all threads are dropped.
///
/// Must be used on [`print_on_exit!`](crate::print_on_exit) because sometimes the threads will drop *after* the main one, corrupting the results.
#[cfg(feature = "enable")]
fn block_until_exited() {
    // Wait for all threads to finish
    #[cfg(feature = "enable")]
    let mut threads = crate::measure::GLOBAL_PROFILER.threads.lock().unwrap();
    #[cfg(feature = "enable")]
    while *threads > 1 {
        threads = crate::measure::GLOBAL_PROFILER.cvar.wait(threads).unwrap();
    }
}

/// Prints the profiled timings to stdout.
///
/// If profiling the `main` function, you can use [`print_on_exit!`](crate::print_on_exit) instead.
///
/// It's recommended to only use it when all threads have exited and have been joined correctly, or you'll risk corrupt data.
#[inline(always)]
pub fn print_timings() -> std::io::Result<()> {
    #[cfg(feature = "enable")]
    crate::measure::GLOBAL_PROFILER.print_timings(std::io::stdout().lock())?;
    Ok(())
}
/// Prints the profiled timings to stderr.
///
/// If profiling the `main` function, you can use [`print_on_exit!`](crate::print_on_exit) instead.
///
/// It's recommended to only use it when all threads have exited and have been joined correctly, or you'll risk corrupt data.
#[inline(always)]
pub fn eprint_timings() -> std::io::Result<()> {
    #[cfg(feature = "enable")]
    crate::measure::GLOBAL_PROFILER.print_timings(std::io::stderr())?;
    Ok(())
}
/// Prints the profiled timings to the provided [`std::io::Write`].
///
/// If profiling the `main` function, you can use [`print_on_exit!`](crate::print_on_exit) instead.
///
/// It's recommended to only use it when all threads have exited and have been joined correctly, or you'll risk corrupt data.
#[inline(always)]
#[allow(unused)]
pub fn print_timings_to(to: impl std::io::Write) -> std::io::Result<()> {
    #[cfg(feature = "enable")]
    crate::measure::GLOBAL_PROFILER.print_timings(to)?;
    Ok(())
}

#[cfg(feature = "nightly")]
#[inline(always)]
pub const fn type_name_of(f: fn()) -> &'static str {
    std::any::type_name_of_val(&f)
}

#[cfg(not(feature = "nightly"))]
#[inline(always)]
pub fn type_name_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}