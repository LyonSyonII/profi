//! Contains all macro items, which should not be used by themselves.
//!
//! Always prefer the macros [`prof!`], [`prof_guard!`] and [`print_on_exit!`].

#[cfg_attr(feature = "enable", derive(Debug))]
pub struct ScopeGuard {
    #[cfg(feature = "enable")]
    instant: std::time::Instant,
}

#[inline(always)]
pub fn dbg_thread() {
    #[cfg(feature = "enable")]
    crate::THREAD_PROFILER.with_borrow(|t| println!("{t:#?}"));
}

pub fn init_profiler() {
    #[cfg(feature = "enable")]
    crate::THREAD_PROFILER.with(|_| ());
}

#[cfg(feature = "enable")]
fn drop_threads() {
    #[cfg(feature = "enable")]
    crate::THREAD_PROFILER.with_borrow_mut(|t| t.set_thread_time());

    #[cfg(all(feature = "rayon", feature = "enable"))]
    {
        // Drop threads manually, as `rayon` never drops them
        let current = std::thread::current().id();

        rayon::broadcast(|t| {
            if std::thread::current().id() != current {
                crate::THREAD_PROFILER.with_borrow_mut(|t| t.manual_drop())
            }
        });
    }
}

/// **Should not be used on its own, will be applied automatically with `print_on_exit!`.**
///
/// Blocks until all threads are dropped.
///
/// Must be used on [`print_on_exit!`] because sometimes the threads will drop *after* the main one, corrupting the results.
#[cfg(feature = "enable")]
fn block_until_exited() {
    // Wait for all threads to finish
    #[cfg(feature = "enable")]
    let mut threads = crate::GLOBAL_PROFILER.threads.lock().unwrap();
    #[cfg(feature = "enable")]
    while *threads > 1 {
        threads = crate::GLOBAL_PROFILER.cvar.wait(threads).unwrap();
    }
}

impl ScopeGuard {
    #[inline(always)]
    #[allow(unused)]
    pub fn new(name: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        #[cfg(feature = "enable")]
        crate::THREAD_PROFILER.with_borrow_mut(|thread| thread.push(name));
        Self {
            #[cfg(feature = "enable")]
            instant: std::time::Instant::now(),
        }
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        #[cfg(feature = "enable")]
        let elapsed = self.instant.elapsed();
        #[cfg(feature = "enable")]
        crate::THREAD_PROFILER.with_borrow_mut(|thread| {
            thread.pop(elapsed);
        })
    }
}

#[allow(dead_code)]
pub struct profiDrop<W: std::io::Write, F: Fn(&mut W)>(W, F);

impl<W, F> profiDrop<W, F>
where
    W: std::io::Write,
    F: Fn(&mut W),
{
    pub fn new(to: W, ondrop: F) -> Self {
        Self(to, ondrop)
    }
}

#[cfg(feature = "enable")]
impl<W, F> std::ops::Drop for profiDrop<W, F>
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

/// Prints the profiled timings to stdout.
///
/// If profiling the `main` function, you can use [`print_on_exit!()`] instead.
///
/// It's recommended to only use it when all threads have exited and have been joined correctly, or you'll risk corrupt data.
#[inline(always)]
pub fn print_timings() -> std::io::Result<()> {
    #[cfg(feature = "enable")]
    crate::GLOBAL_PROFILER.print_timings(std::io::stdout().lock())?;
    Ok(())
}
/// Prints the profiled timings to stderr.
///
/// If profiling the `main` function, you can use [`print_on_exit!()`] instead.
///
/// It's recommended to only use it when all threads have exited and have been joined correctly, or you'll risk corrupt data.
#[inline(always)]
pub fn eprint_timings() -> std::io::Result<()> {
    #[cfg(feature = "enable")]
    crate::GLOBAL_PROFILER.print_timings(std::io::stderr())?;
    Ok(())
}
/// Prints the profiled timings to the provided [`std::io::Write`].
///
/// If profiling the `main` function, you can use [`print_on_exit!()`] instead.
///
/// It's recommended to only use it when all threads have exited and have been joined correctly, or you'll risk corrupt data.
#[inline(always)]
#[allow(unused)]
pub fn print_timings_to(to: impl std::io::Write) -> std::io::Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(500));
    #[cfg(feature = "enable")]
    crate::GLOBAL_PROFILER.print_timings(to)?;
    Ok(())
}
