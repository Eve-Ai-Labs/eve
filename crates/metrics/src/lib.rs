pub use metric_exporter::Metrics;
use opentelemetry::{
    global::meter,
    metrics::{Counter, Histogram, Meter, UpDownCounter},
};
use std::sync::LazyLock;

mod metric_exporter;

static METER: LazyLock<Meter> = LazyLock::new(|| meter(module_path!()));

pub static ERRORS: LazyLock<Counter<u64>> = LazyLock::new(|| METER.u64_counter("errors").build());
pub static REQUESTS: LazyLock<Counter<u64>> =
    LazyLock::new(|| METER.u64_counter("requests").build());
pub static PROCESSING: LazyLock<UpDownCounter<i64>> =
    LazyLock::new(|| METER.i64_up_down_counter("processing").build());
pub static TIMEOUTS: LazyLock<Counter<u64>> =
    LazyLock::new(|| METER.u64_counter("timeouts").build());
pub static LATENCY: LazyLock<Histogram<u64>> =
    LazyLock::new(|| METER.u64_histogram("latency").build());
