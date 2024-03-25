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
