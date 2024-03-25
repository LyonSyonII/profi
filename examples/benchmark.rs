// Run with 'cargo run --release --example benchmark --features metaprof > out'

fn main() {
    profi::print_on_exit!();

    // Benchmark how much time it takes for `prof!` to create and drop
    for _ in 0..100 {
        bench()
    }

}

fn bench() {
    {
        // Get function name
        profi::prof!();
    }
    {
        // Given str
        profi::prof!("prof_given_str");
    }
    {
        // Given name
        profi::prof!(prof_given_name);
    }
    {
        // Dynamic name
        profi::prof!(fmt = "prof_{:?}", &() as *const () as u16);
    }
    {
        // With guard
        let _guard = profi::prof_guard!(prof_guard);
    }
    {
        // Time Self
        for _ in 0..1000 {
            profi::prof!(self);
            profi::prof!(_self);
        }
    }

    // High number of calls
    // 10..100_000
    let mut iter = 10;
    for _ in 0..5 {
            profi::prof!(fmt = "prof{iter}");
            miniprof::prof!(fmt = "prof{iter}");
        }
        iter *= 10;
    

    // Highly nested
    fn nest(depth: usize, limit: usize) {
        if depth > limit {
            return;
        
        profi::prof!(fmt = "depth = {depth}");
        miniprof::prof!(fmt = "depth = {depth}");
        nest(depth + 1, limit);
    }
    nest(0, 1000);

    // Very large amount of leaves
    {
        profi::prof!("[leaves]");
        for i in 0..10_000 {
            profi::prof!(fmt = "[leaves] i = {i}");
        }
    }
}
