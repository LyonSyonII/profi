use profi::print_on_exit;

fn main() {
    print_on_exit!();
    
    profi::prof_block!{
        // 1s
        sleep_std(1_000_000)
    }
    profi::prof_block!{
        // 100µs
        sleep_std(100_000)
    }
    profi::prof_block!{
        // 10µs
        sleep_std(10_000)
    }
    profi::prof_block!{
        // 1µs
        sleep_std(1000)
    }
    profi::prof_block!{
        // 100ns
        sleep_std(100)
    }
    profi::prof_block!{
        // 10ns
        sleep_std(10)
    }
    profi::prof_block!{
        // 1ns
        sleep_std(1)
    }
    
    let sleep_spin = |nanos| {
        let start = minstant::Instant::now();
        let duration = std::time::Duration::from_nanos(nanos);
        while start.elapsed() < duration {
            // std::hint::spin_loop();
        }
    };

    profi::prof_block!{
        // 1s
        sleep_spin(1_000_000)
    }
    profi::prof_block!{
        // 100µs
        sleep_spin(100_000)
    }
    profi::prof_block!{
        // 10µs
        sleep_spin(10_000)
    }
    profi::prof_block!{
        // 1µs
        sleep_spin(1000)
    }
    profi::prof_block!{
        // 100ns
        sleep_spin(100)
    }
    profi::prof_block!{
        // 10ns
        sleep_spin(10)
    }
    profi::prof_block!{
        // 1ns
        sleep_spin(1)
    }
    
    unsafe {
        let pre = core::arch::x86_64::_rdtsc();
        for _ in 0..1_000_000 {
            std::arch::asm! {
                "nop",
                options(raw),
            };
        }
        let post = core::arch::x86_64::_rdtsc();
        let cycles = (post - pre) / 1_000_000;
        dbg!(cycles);
    }
}

fn sleep_std(nanos: u64) {
    std::thread::sleep(std::time::Duration::from_nanos(nanos))
}