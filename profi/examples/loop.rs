use profi::{print_on_exit, prof};

fn main() {
    print_on_exit!();

    for _ in 0..100 {
        prof!(iteration);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
