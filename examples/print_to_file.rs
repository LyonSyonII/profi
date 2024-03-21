use miniprof::{print_on_exit, prof};

fn main() -> std::io::Result<()> {
    let mut file = std::fs::File::create("./print_to_file_results.txt")?;
    print_on_exit!(to = &mut file);

    std::thread::scope(|s| {
        prof!(spawn_threads);
        s.spawn(|| {
            prof!(thread1);
            wait(100);
        });
        s.spawn(|| {
            prof!(thread2);
            wait(200);
        });
        s.spawn(|| {
            prof!(thread3);
            wait(300);
        });
    });

    Ok(())
}

fn wait(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms))
}
