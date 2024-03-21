fn main() {
    miniprof::print_on_exit!();

    // Benchmark how much time it takes for `prof!` to create and drop

    {
        // Get function name
        miniprof::prof!();
    }
    {
        // Given str
        miniprof::prof!("prof_given_str");
    }
    {
        // Given name
        miniprof::prof!(prof_given_name);
    }
    {
        // Dynamic name
        miniprof::prof!(fmt = "prof_{:?}", &() as *const () as u16);
    }
    {
        // With guard
        let _guard = miniprof::prof_guard!(prof_guard);
    }
    
    // Many times
    // 10..100_000
    let mut iter = 10;
    for _ in 0..5 {
        for _ in 0..iter {
            miniprof::prof!(fmt = "prof{iter}");
        }
        iter *= 10;
    }
    
    // Highly nested
    fn nest(depth: usize, limit: usize) {
        if depth > limit {
            return
        }
        miniprof::prof!(fmt = "depth = {depth}");
        nest(depth + 1, limit);
    }
    nest(0, 20);
}