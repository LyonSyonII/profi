use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

fn main() {
    profi::print_on_exit!();
    
    // Benchmark different implementations of adding up the numbers in a sequence.
    // Collect the range in a Vec to avoid optimizations.
    const N: usize = 1 << 32;
    let range = (0..N).collect::<Vec<_>>();

    let result = {
        profi::prof!("sequential");
        range.iter().sum::<usize>() 
    };
    
    let reduction = true;
    let sum = true;
    let manual = true;

    if reduction {
        profi::prof!("rayon-reduction");
        let sum: usize = range.par_iter().copied().reduce(|| 0, |a, b| a + b);
        assert_eq!(sum, result);
    }

    if sum {
        profi::prof!("rayon-sum");
        let sum: usize = range.par_iter().sum();
        assert_eq!(sum, result);
    }

    if manual {
        profi::prof!("manual-blocks");
        let num_threads: usize = std::thread::available_parallelism().unwrap().into();
        let sum = std::sync::atomic::AtomicUsize::new(0);
        std::thread::scope(|s| {
            let sum = &sum;
            for block in range.chunks(num_threads) {
                s.spawn(move || {
                    sum.fetch_add(block.iter().sum(), std::sync::atomic::Ordering::Relaxed);
                });
            }
        });
        assert_eq!(sum.into_inner(), result);
    }
    
    println!("{result}");
}