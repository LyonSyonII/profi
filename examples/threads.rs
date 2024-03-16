use miniprof::{print_on_exit, prof};

fn main() {
    print_on_exit!();
    
    std::thread::scope(|s| {
        for i in 0..10 {
            s.spawn(move || {
                for _ in 0..100 {
                    let _guard = if i < 6 {
                        prof!("6 first threads")
                    } else {
                        prof!("4 last threads")
                    };
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            });
        }
    });
}
