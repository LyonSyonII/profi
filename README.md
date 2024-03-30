# Profi

A simple profiler for single and multithreaded applications.

Record the time it takes for a scope to end and print the timings when the program exits.

Each measurement has an overhead of ~25ns-50ns, so it shouldn't impact benchmarks.  
Run the [benchmarks](https://github.com/LyonSyonII/profi/blob/main/examples/benchmark.rs) example to see what's the overhead on your machine.

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

See the [`examples`](https://github.com/LyonSyonII/profi/tree/main/examples) for more usage cases.

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
```plaintext
┌──────────────┬────────────────────┬───────────┬────────────┬──────────┬──────────────┬───────┐
│ Name         ┆ % Application Time ┆ Real Time ┆ % CPU Time ┆ CPU Time ┆ Average time ┆ Calls │
╞══════════════╪════════════════════╪═══════════╪════════════╪══════════╪══════════════╪═══════╡
│ simple::main ┆ 100.00%            ┆ 200.13ms  ┆      -     ┆     -    ┆       -      ┆     1 │
└──────────────┴────────────────────┴───────────┴────────────┴──────────┴──────────────┴───────┘
```

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
```plaintext
┌────────────┬────────────────────┬───────────┬────────────┬──────────┬──────────────┬───────┐
│ Name       ┆ % Application Time ┆ Real Time ┆ % CPU Time ┆ CPU Time ┆ Average time ┆ Calls │
╞════════════╪════════════════════╪═══════════╪════════════╪══════════╪══════════════╪═══════╡
│ loop::main ┆ 100.00%            ┆ 1.01s     ┆      -     ┆     -    ┆       -      ┆     1 │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ iteration  ┆ 99.99%             ┆ 1.01s     ┆      -     ┆     -    ┆ 10.10ms/call ┆   100 │
└────────────┴────────────────────┴───────────┴────────────┴──────────┴──────────────┴───────┘
```

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
```plaintext
┌───────────┬────────────────────┬───────────┬────────────┬──────────┬──────────────┬───────┐
│ Name      ┆ % Application Time ┆ Real Time ┆ % CPU Time ┆ CPU Time ┆ Average time ┆ Calls │
╞═══════════╪════════════════════╪═══════════╪════════════╪══════════╪══════════════╪═══════╡
│ main      ┆ 100.00%            ┆ 1.01s     ┆      -     ┆     -    ┆       -      ┆     1 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│  6 first  ┆ 99.98%             ┆ 1.01s     ┆ 54.55%     ┆ 6.04s    ┆ 10.08ms/call ┆   600 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│  4 last   ┆ 99.96%             ┆ 1.01s     ┆ 36.36%     ┆ 4.03s    ┆ 10.07ms/call ┆   400 │
└───────────┴────────────────────┴───────────┴────────────┴──────────┴──────────────┴───────┘
```
"CPU Time" is the combined time all threads have spent on that scope.  

For example, "6 first" has a "CPU Time" of 6 seconds because each thread waits 1 second, and the program spawns six of them.

## Attribute
```rust
use profi::{prof, print_on_exit};


# Features

| Name             | Description                                                                                                                                                                            |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `enable`         | Activates the profiling, if not active all macros become no-ops                                                                                                                        |
| `attributes`     | Enables the `#[prof]` macro                                                                                                                                                            |
| `deep-hierarchy` | By default `profi` merges all uses of a function, use this feature to disable this behaviour.<br/>See the [`nested` example for more information](https://github.com/lyonsyonii/profi) |
| `rayon`          | Necessary if using [`rayon`](https://crates.io/crates/rayon)                                                                                                                           |
