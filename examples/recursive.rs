use profi::print_on_exit;

fn main() {
    print_on_exit!();

    fn rec(depth: usize, limit: usize) {
        if depth > limit {
            return;
        }
        // As you can see, each instance of "rec" is treated as a different measurement
        // This is because `profi` is hierarchical, and in this case
        profi::prof!();
        rec(depth + 1, limit);
    }
    rec(0, 5);
}