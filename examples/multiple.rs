use miniprof::{print_on_exit, prof};

fn main() {
    // Prints the timings to stdout when the program exits
    // Always put at the top of the main function to ensure it's the last thing to run
    print_on_exit!();

    // The `prof!` macro creates a guard that records the time until it goes out of scope
    prof!(main);

    // Sleep for a bit to simulate some work
    std::thread::sleep(std::time::Duration::from_millis(200));
}
