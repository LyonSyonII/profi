#[profi::main]
fn main() {
    profiled(
        Struct {
            values: vec![1, 2, 3, 4],
        },
        Tuple("hey"),
    );
}

struct Struct<T> {
    values: Vec<T>,
}

struct Tuple<'a>(&'a str);

// Really contrived signature to ensure macro works properly
#[profi::profile]
fn profiled<'a, T: Default + Clone>(
    Struct { values }: Struct<T>,
    Tuple(tuple): Tuple<'a>,
) -> (T, &'a str) {
    (values.first().cloned().unwrap_or_default(), tuple)
}
