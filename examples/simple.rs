use miniprof::{print_on_exit, prof};

fn main() {
    // Prints the timings to stdout when the program exits
    // Always put at the top of the main function to ensure it's the last thing to run
    //
    // An implicit `main` guard is always created, try running this example to see it!
    print_on_exit!();

    // Sleep for a bit to simulate some work
    std::thread::sleep(std::time::Duration::from_millis(200));
}
