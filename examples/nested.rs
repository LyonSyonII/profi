use miniprof::{print_on_exit, prof};

fn main() {
    print_on_exit!();

    a();
}

fn a() {
    // If `prof` is called without any argument, `miniprof` will do its best to get the function's name
    prof!();
    std::thread::sleep(std::time::Duration::from_millis(10));

    b();
}

fn b() {
    prof!();
    std::thread::sleep(std::time::Duration::from_millis(10));

    c();
}

fn c() {
    prof!();
    std::thread::sleep(std::time::Duration::from_millis(10));

    d();
}

fn d() {
    prof!();
    std::thread::sleep(std::time::Duration::from_millis(10));
}