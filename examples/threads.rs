use miniprof::{print_on_exit, prof};

fn do_work(i: usize) {
    for _ in 0..100 {
        // Need to bind it to a variable to ensure it doesn't go out of scope
        let _guard = if i < 6 {
            prof!("6 first threads")
        } else {
            prof!("4 last threads")
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
