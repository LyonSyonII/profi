use miniprof::{print_on_exit, prof_guard};

fn sleep(time: u64) {
    // Must be saved into an explicit guard, or it will be dropped at the end of the `if` block
    let _guard = if time < 100 {
        prof_guard!("< 100")
    } else {
        prof_guard!(">= 100")
    };
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn main() {
    print_on_exit!();

    sleep(50);
    sleep(150);
}
