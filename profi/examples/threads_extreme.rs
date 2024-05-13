use profi::{print_on_exit, prof, prof_guard};

fn do_work(i: usize) {
    // Need to bind it to a variable to ensure it isn't dropped before sleeping
    let _guard = if i < 600 {
        prof_guard!("600 first")
    } else {
        prof_guard!("400 last")
    };
    std::thread::sleep(std::time::Duration::from_millis(10));
    // The guard goes out of scope here

}

fn main() {
    print_on_exit!();
    
    for _ in 0..100 {
        // Spawn 1000 threads
        std::thread::scope(|s| {
            prof!(scope);
            for i in 0..1000 {
                s.spawn(move || {
                    do_work(i);
                });
            }
        })
    }
}
