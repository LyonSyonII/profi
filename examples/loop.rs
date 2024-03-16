use miniprof::{print_on_exit, prof};

fn main() {
    print_on_exit!();

    for _ in 0..100 {
        prof!(loop);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
