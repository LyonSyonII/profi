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
    let main1 = prof!("main1");

    // Sleep for a bit to simulate some work
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Call a function that has its own profiling
    wait_for_a_bit();
    wait_for_a_bit();

    // Drop the `main1` guard
    drop(main1);

    // Create a new guard that will end when the program ends
    prof!(main2);
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Total time is calculated by adding up all the time spent in the `prof!` guards
    // This will print something like:
    // +----------------+--------------------+-----------+------------+----------+---------------+-------+
    // | Name           | % Application Time | Real Time | % CPU Time | CPU Time | Average time  | Calls |
    // +----------------+--------------------+-----------+------------+----------+---------------+-------+
    // | main           | 100.00%            | 600.43ms  | -          | -        | -             | -     |
    // +----------------+--------------------+-----------+------------+----------+---------------+-------+
    // | main1          | 66.67%             | 400.33ms  | -          | -        | 400.33ms/call |     1 |
    // +----------------+--------------------+-----------+------------+----------+---------------+-------+
    // | wait_for_a_bit | 33.35%             | 200.23ms  | -          | -        | 100.12ms/call |     2 |
    // +----------------+--------------------+-----------+------------+----------+---------------+-------+
    // | main2          | 33.33%             | 200.10ms  | -          | -        | 200.10ms/call |     1 |
    // +----------------+--------------------+-----------+------------+----------+---------------+-------+
    //
    // Here we can see the implicit `main` guard, along with `main1` and `main2`.
    //
    // The '% Application Time' values do not add up to 100% because the profiling is hierarchical.
    // `wait_for_a_bit` is contained in the `main1` scope, so its `33.35%` accounts for the total of that guard.
}
