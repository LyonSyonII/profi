# Profi

A simple profiler for single and multithreaded applications.

Record the time it takes for a scope to end and print the timings when the program exits.

Each measurement has an overhead of ~25ns-50ns, so it shouldn't impact benchmarks.  
Run the [benchmarks](https://github.com/LyonSyonII/profi/blob/main/profi/examples/benchmark.rs) example to see what's the overhead on your machine.

# Setup

`profi` is controlled by the `enable` feature, which is active by default.  
When disabled, all macros and methods will become no-ops, resulting in zero impact on your code.

To disable it, add `default-features = false` to the `profi` dependency in your `Cargo.toml`.

For convenience, you can also add a custom feature:
```toml
[dependencies]
profi = { version = "*", default-features = false }

[features]
prof = ["profi/enable"]
```

And run it with `cargo run --release --features prof`

**If you use [`rayon`](https://crates.io/crates/rayon), enable the [`rayon` feature](#features)!**

# Usage

See the [`examples`](https://github.com/LyonSyonII/profi/tree/main/profi/examples) for more usage cases.

## Basic Usage
```rust
use profi::{prof, print_on_exit};

fn main() {
 // Prints the timings to stdout when the program exits
 // Always put at the top of the main function to ensure it's dropped last
 //
 // An implicit `main` guard is created to profile the whole application
 print_on_exit!();

 // Sleep for a bit to simulate some work
 std::thread::sleep(std::time::Duration::from_millis(200));
}
```
![codeimage-snippet_30](https://github.com/LyonSyonII/profi/assets/69039201/f6aaf5ad-7ae7-4371-aae5-753df3fdbfcd)

## Loops
```rust
use profi::{prof, print_on_exit};

fn main() {
  print_on_exit!();

  for _ in 0..100 {
      prof!(iteration);
      std::thread::sleep(std::time::Duration::from_millis(10));
  }
}
```
![codeimage-snippet_31](https://github.com/LyonSyonII/profi/assets/69039201/e7ef500d-6a42-42ae-baf9-e87f35029b4c)

## Multiple threads
```rust
use profi::{print_on_exit, prof_guard};

fn do_work(i: usize) {
    // Need to bind it to a variable to ensure it isn't dropped before sleeping
    let _guard = if i < 6 {
        prof_guard!("6 first")
    } else {
        prof_guard!("4 last")
    };
    std::thread::sleep(std::time::Duration::from_millis(10));
    // The guard goes out of scope here
}

fn main() {
    print_on_exit!();
    
    // Spawn 10 threads
    std::thread::scope(|s| {
        for i in 0..10 {
            s.spawn(move || {
                do_work(i);
            });
        }
    });
}
```

![codeimage-snippet_32](https://github.com/LyonSyonII/profi/assets/69039201/b1471e82-25c2-4c56-a9a0-9dd9a2b7d005)

"CPU Time" is the combined time all threads have spent on that scope.  

For example, "6 first" has a "CPU Time" of 60 milliseconds because each thread waits 10ms, and the program spawns six of them.

## Attribute
Enable the `attributes` feature to use the `profile` attribute on functions.  
This will add a guard at the start of the function.

```ignore
use profi::profile;

#[profile]
fn anotated() { /* ... */ }
```

# Features

| Name             | Description                                                                                                                                                                                                     |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `enable`         | Activates the profiling, if not active all macros become no-ops                                                                                                                                                 |
| `attributes`     | Enables the `#[prof]` macro                                                                                                                                                                                     |
| `deep-hierarchy` | By default `profi` merges all uses of a function, use this feature to disable this behaviour.<br/>See the [`nested` example for more information](https://github.com/LyonSyonII/profi/tree/main/profi/examples) |
| `nightly`        | Enables nightly-only optimizations (unused at the moment)                                                                                                                                                       |
| `rayon`          | Necessary if using [`rayon`](https://crates.io/crates/rayon)                                                                                                                                                    |
