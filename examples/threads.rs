use profi::{print_on_exit, prof_guard};

fn do_work(i: usize) {
    for _ in 0..100 {
        // Need to bind it to a variable to ensure it doesn't go out of scope
        let _guard = if i < 6 {
            prof_guard!("6 first")
        } else {
            prof_guard!("4 last")
        };
        std::thread::sleep(std::time::Duration::from_millis(10));
        // The guard goes out of scope here
    }
}

fn main() {
    print_on_exit!();

    std::thread::scope(|s| {
        for i in 0..10 {
            s.spawn(move || {
                do_work(i);
            });
        }
    });
}
