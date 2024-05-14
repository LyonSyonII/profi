use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

fn main() {
    profi::print_on_exit!();
    
    // Benchmark different implementations of adding up the numbers in a sequence.
    // Collect the range in a Vec to avoid optimizations.
    type Int = u32;
    type AtomicInt = std::sync::atomic::AtomicU32;
    const N: Int = Int::MAX;

    let range = { 
        profi::prof!("range creation");
        (0..N).collect::<Vec<_>>() 
    };

    let result = {
        profi::prof!("sequential");
        range.iter().sum() 
    };
    
    let reduction = true;
    let sum = true;
    let manual = true;

    if reduction {
        profi::prof!("rayon-reduction");
        let sum = range.par_iter().copied().reduce(|| 0, |a, b| a + b);
        assert_eq!(sum, result);
    }

    if sum {
        profi::prof!("rayon-sum");
        let sum = range.par_iter().sum::<Int>();
        assert_eq!(sum, result);
    }

    if manual {
        profi::prof!("manual-blocks");
        let num_threads: usize = std::thread::available_parallelism().unwrap().into();
        let sum = AtomicInt::new(0);
        std::thread::scope(|s| {
            let sum = &sum;
            let range = &range;
            for t in 0..num_threads {
                s.spawn(move || {
                    // TODO: This causes a Deadlock
                    profi::prof!("manual-blocks-iter");
                    let bs = (N as usize / num_threads) as usize;
                    let rest = (N as usize % num_threads) as usize;
                    // Add 1 element to each thread until `rest`
                    let start = if t < rest {
                        bs * t + t
                    } else {
                        // When all extra elements have been added, adapt `start` to this extra elements
                        bs * t + rest
                    } as usize;
                    let end = start + bs + (t < rest) as usize;
                    
                    let local_sum = range[start..end].iter().sum();
                    sum.fetch_add(local_sum, std::sync::atomic::Ordering::AcqRel);
                });
            }
        });
        assert_eq!(sum.into_inner(), result);
    }
}