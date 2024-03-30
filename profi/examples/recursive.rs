use profi::print_on_exit;

fn main() {
    // Try running with 'cargo run --release --example benchmark --features deep-hierarchy'
    // for a look on how the feature works

    print_on_exit!();

    fn rec(depth: usize, limit: usize) {
        if depth > limit {
            return;
        }
        // If 'deep-hierarchy' is enabled, each call will go deeper in the hierarchy
        // If not (default), all calls will be merged into one
        profi::prof!();
        rec(depth + 1, limit);
    }
    rec(0, 5);
}
