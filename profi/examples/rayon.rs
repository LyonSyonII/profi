use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() {
    profi::print_on_exit!();
    const N: usize = 1_000_000;
    let result = {
        profi::prof!("base-case");
        std::hint::black_box(0..N).sum::<usize>() 
    };
    
    {
        profi::prof!("atomic");
        let sum = std::sync::atomic::AtomicUsize::new(0);
        (0..N).into_par_iter().for_each(|i| {
            sum.fetch_add(i, std::sync::atomic::Ordering::Relaxed);
        });
        assert_eq!(sum.into_inner(), result);
    }

    {
        profi::prof!("reduction");
        let sum: usize = (0..N).into_par_iter().reduce(|| 0, |a, b| {
            a + b 
        });
        assert_eq!(sum, result);
    }

    {
        profi::prof!("sum");
        let sum: usize = (0..N).into_par_iter().sum();
        assert_eq!(sum, result);
    }
}