//! Contains all macro items, which should not be used by themselves.
//! 
//! Always prefer the macros [`prof!`] and [`print_on_exit!`].

#[cfg_attr(feature = "enable", derive(Debug))]
pub struct ScopeGuard {
    #[cfg(feature = "enable")]
    instant: std::time::Instant,
}

impl ScopeGuard {
    pub fn new(name: &'static str) -> Self {
        #[cfg(feature = "enable")]
        crate::THREAD_PROFILER.with_borrow_mut(|thread| thread.push(name));
        Self {
            #[cfg(feature = "enable")]
            instant: std::time::Instant::now(),
        }
    }
}

#[cfg(feature = "enable")]
impl Drop for ScopeGuard {
    fn drop(&mut self) {
        crate::THREAD_PROFILER.with_borrow_mut(|thread| {
            thread.pop(self.instant.elapsed());
        })
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
    crate::GLOBAL_PROFILER.print_timings(&mut std::io::stdout().lock())?;
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
    crate::GLOBAL_PROFILER.print_timings(&mut std::io::stderr())?;
    Ok(())
}
/// Prints the profiled timings to the provided [`std::io::Write`].
///
/// If profiling the `main` function, you can use [`print_on_exit!()`] instead.
/// 
/// It's recommended to only use it when all threads have exited and have been joined correctly, or you'll risk corrupt data.
#[inline(always)]
pub fn print_timings_to(to: &mut impl std::io::Write) -> std::io::Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(500));
    #[cfg(feature = "enable")]
    crate::GLOBAL_PROFILER.print_timings(to)?;
    Ok(())
}