use miniprof::{print_on_exit, prof};

fn wait_for_a_bit() {
    prof!(wait_for_a_bit);
    std::thread::sleep(std::time::Duration::from_millis(100));
}

fn main() {
    // Prints the timings to stdout when the program exits
    // Always put at the top of the main function to ensure it's the last thing to run
    print_on_exit!();

    // The `prof!` macro creates a guard that records the time until it goes out of scope
    let _main = prof!("main");

    // Sleep for a bit to simulate some work
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Call a function that has its own profiling
    wait_for_a_bit();
    wait_for_a_bit();

    prof!(main2);
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Total time is calculated by adding up all the time spent in the `prof!` guards
    // This will print something like:
    // +----------------+-----------------+---------------+-------+
    // | Name           | % of total time | Average time  | Calls |
    // +----------------+-----------------+---------------+-------+
    // | main           | 66.66%          | 400.39ms/call |     1 |
    // +----------------+-----------------+---------------+-------+
    // | wait_for_a_bit | 33.34%          | 100.15ms/call |     2 |
    // +----------------+-----------------+---------------+-------+
    //
    // `main` took 66.66% of the time because hierarchy is not taken into account:
    // 400ms / (400ms + 100ms + 100ms) = 66.66%
    //
    // If this behavior is not desired, profile sections of the code separately
}
