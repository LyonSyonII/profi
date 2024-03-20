fn main() {
    miniprof::print_on_exit!();

    let clear_cache = vec![0u8; u32::MAX as usize];
    let clear_cache = || {
        miniprof::prof!(clear_cache);
        for i in &clear_cache {
            std::hint::black_box(i);
        }
    };

    // Benchmark how much time it takes for `prof!` to create and drop

    {
        // Get function name
        miniprof::prof!();
    }
    {
        // Given str
        let _guard = miniprof::prof!("prof_given_str");
    }
    {
        // Given name
        miniprof::prof!(prof_given_name);
    }
    
    let mut iter = 10;

    for _ in 0..iter {
        miniprof::prof!(prof10);
    }
    
    iter *= 10;
    
    for _ in 0..iter {
        miniprof::prof!(prof100);
    }
    
    iter *= 10;
    
    for _ in 0..iter {
        miniprof::prof!(prof1000);
    }

    iter *= 10;

    for _ in 0..iter {
        miniprof::prof!(prof10_000);
    }

    iter *= 10;

    for _ in 0..iter {
        miniprof::prof!(prof100_000);
    }
}