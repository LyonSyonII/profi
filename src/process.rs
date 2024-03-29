#[cfg(feature = "enable")]
#[derive(Debug, Clone)]
struct Timing {
    name: String,
    /// % Application Time
    percent_app: f64,
    /// Real Time
    total_real: std::time::Duration,
    /// % CPU Time
    percent_cpu: f64,
    /// CPU Time
    total_cpu: std::time::Duration,
    average: std::time::Duration,
    calls: usize,
}

#[cfg(feature = "enable")]
impl Timing {
    fn from_durations(
        name: String,
        timings: &[std::time::Duration],
        total: std::time::Duration,
    ) -> Self {
        let sum = timings.iter().sum::<std::time::Duration>();
        let percent = if !total.is_zero() {
            (sum.as_secs_f64() / total.as_secs_f64()) * 100.0
        } else {
            100.0
        };
        let average = sum / timings.len().max(1) as u32;
        Self {
            name,
            percent_app: percent,
            total_real: sum,
            percent_cpu: percent,
            total_cpu: sum,
            average,
            calls: timings.len(),
        }
    }
    fn merge(&mut self, other: &Timing) {
        self.average = (self.average + other.average) / 2;
        self.calls += other.calls;
        self.total_cpu += other.total_cpu;
        self.total_real = self.total_real.max(other.total_real);
    }
    fn update_percent(&mut self, total_app: std::time::Duration, total_cpu: std::time::Duration) {
        self.percent_app = (self.total_real.as_secs_f64() / total_app.as_secs_f64()) * 100.;
        self.percent_cpu = (self.total_cpu.as_secs_f64() / total_cpu.as_secs_f64()) * 100.;
    }
}

#[cfg(feature = "enable")]
fn create_table(timings: Vec<Timing>) -> comfy_table::Table {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header([
        "Name",
        "% Application Time",
        "Real Time",
        "% CPU Time",
        "CPU Time",
        "Average time",
        "Calls",
    ]);

    let empty = || comfy_table::Cell::new("-").set_alignment(comfy_table::CellAlignment::Center);

    for timing in timings {
        fn cell(c: impl Into<comfy_table::Cell>) -> comfy_table::Cell {
            c.into()
        }

        let name = cell(timing.name);
        let app_percent = cell(format!("{:.2}%", timing.percent_app));
        let real_time = cell(format!("{:.2?}", timing.total_real));
        let (cpu_percent, cpu_time) = if timing.total_real == timing.total_cpu {
            (empty(), empty())
        } else {
            (
                cell(format!("{:.2}%", timing.percent_cpu)),
                cell(format!("{:.2?}", timing.total_cpu)),
            )
        };
        let average = if timing.average.is_zero() || timing.calls <= 1 {
            empty()
        } else {
            cell(format!("{:.2?}/call", timing.average))
        };
        let calls = if timing.calls == 0 {
            empty()
        } else {
            cell(timing.calls).set_alignment(comfy_table::CellAlignment::Right)
        };
        table.add_row([
            name,
            app_percent,
            real_time,
            cpu_percent,
            cpu_time,
            average,
            calls,
        ]);
    }

    table
}

#[derive(Debug, Clone)]
struct Node<'a> {
    timings: Vec<std::time::Duration>,
    children: indexmap::IndexMap<&'a str, Node<'a>>,
    depth: usize,
}

impl<'a> Node<'a> {
    fn new(depth: usize) -> Self {
        Self {
            timings: Vec::new(),
            children: indexmap::IndexMap::new(),
            depth
        }
    }
}

pub fn print_timings(threads: &[(std::time::Duration, Vec<crate::measure::Measure>)]) -> std::io::Result<()> {
    let mut tree = indexmap::IndexMap::new();
    for (time, measurements) in threads {
        tree = create_tree(measurements, tree);
    }
    // println!("{:#?}", tree);
    Ok(())
}

fn get_current<'r, 'node>(current_path: &[usize], tree: &'r mut indexmap::IndexMap<&str, Node<'node>>) -> Option<&'r mut Node<'node>> {
    let (_, mut current) = tree.get_index_mut(*current_path.first()?)?;
    for c in current_path.get(1..).unwrap_or_default().iter().copied() {
        (_, current) = current.children.get_index_mut(c)?;
    }
    Some(current)
}

fn create_tree<'node, 'm: 'node>(measurements: &'m [crate::measure::Measure], mut tree: indexmap::IndexMap<&'m str, Node<'node>>) -> indexmap::IndexMap<&'m str, Node<'node>> {
    let mut current_path: Vec<usize> = Vec::new();
    let mut start_times: Vec<minstant::Instant> = Vec::new();
    
    for m in measurements {
        match m.ty {
            crate::measure::MeasureType::Start { ref name } => {
                let name = name.as_ref();
                start_times.push(m.time);

                let Some(current) = get_current(&current_path, &mut tree) else {
                    // No current subtree, so insert to root
                    if let Some(idx) = tree.get_index_of(name) {
                        current_path.push(idx);
                    } else {
                        tree.insert(name, Node::new(0));
                        current_path.push(tree.len()-1);
                    }
                    continue;
                };
                // Insert node as child of current
                if let Some(idx) = current.children.get_index_of(name) {
                    current_path.push(idx);
                } else {
                    current.children.insert(name, Node::new(current.depth + 1));
                    current_path.push(current.children.len()-1);
                }
            },
            crate::measure::MeasureType::End => {
                let current = get_current(&current_path, &mut tree).expect("[profi] 'pop' called and 'current' is 'None', this should never happen!");
                let start = start_times.pop().expect("[profi] 'pop' called and 'start_times' is empty, this should never happen!");
                current.timings.push(m.time.duration_since(start));
                current_path.pop();
            },
        }
    }

    tree
}