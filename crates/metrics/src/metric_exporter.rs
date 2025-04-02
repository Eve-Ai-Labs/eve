use async_trait::async_trait;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    metrics::{
        data::{Histogram, ResourceMetrics, Sum},
        exporter::PushMetricExporter,
        Temporality,
    },
};
use std::sync::{
    atomic::{AtomicI64, AtomicU64, Ordering},
    Arc,
};
use tracing::error;
use types::cluster::MetricsInfo;

#[derive(Default)]
struct Inner {
    requests: AtomicU64,
    processing: AtomicI64,
    timeouts: AtomicU64,
    errors: AtomicU64,
    latency_sum: AtomicU64,
    latency_count: AtomicU64,
}

#[derive(Default, Clone)]
pub struct Metrics {
    inner: Arc<Inner>,
}

#[async_trait]
impl PushMetricExporter for Metrics {
    async fn export(&self, metrics: &mut ResourceMetrics) -> OTelSdkResult {
        let iter = metrics
            .scope_metrics
            .iter()
            .flat_map(|scope_metrics| scope_metrics.metrics.iter());

        for item in iter {
            match item.name.as_ref() {
                "requests" => {
                    if let Some(data) = item.data.as_any().downcast_ref::<Sum<u64>>() {
                        let sum: u64 = data.data_points.iter().map(|p| p.value).sum();
                        self.inner.requests.fetch_add(sum, Ordering::Relaxed);
                    } else {
                        error!("Invalid data type for metric 'requests'");
                    }
                }
                "processing" => {
                    if let Some(data) = item.data.as_any().downcast_ref::<Sum<i64>>() {
                        let sum: i64 = data.data_points.iter().map(|p| p.value).sum();
                        self.inner.processing.store(sum, Ordering::Relaxed);
                    } else {
                        error!("Invalid data type for metric 'processing'");
                    }
                }
                "timeouts" => {
                    if let Some(data) = item.data.as_any().downcast_ref::<Sum<u64>>() {
                        let sum: u64 = data.data_points.iter().map(|p| p.value).sum();
                        self.inner.timeouts.fetch_add(sum, Ordering::Relaxed);
                    } else {
                        error!("Invalid data type for metric 'timeouts'");
                    }
                }
                "latency" => {
                    if let Some(data) = item.data.as_any().downcast_ref::<Histogram<u64>>() {
                        let (sum, count) = data
                            .data_points
                            .iter()
                            .fold((0, 0), |(sum, count), p| (sum + p.sum, count + p.count));
                        self.inner.latency_sum.fetch_add(sum, Ordering::Relaxed);
                        self.inner.latency_count.fetch_add(count, Ordering::Relaxed);
                    } else {
                        error!("Invalid data type for metric 'latency'");
                    }
                }
                "errors" => {
                    if let Some(data) = item.data.as_any().downcast_ref::<Sum<u64>>() {
                        let sum: u64 = data.data_points.iter().map(|p| p.value).sum();
                        self.inner.errors.fetch_add(sum, Ordering::Relaxed);
                    } else {
                        error!("Invalid data type for metric 'errors'");
                    }
                }
                _ => {
                    error!("unknown metric: {}", item.name);
                }
            }
        }
        Ok(())
    }

    async fn force_flush(&self) -> OTelSdkResult {
        self.inner
            .requests
            .store(Default::default(), Ordering::Relaxed);
        self.inner
            .processing
            .store(Default::default(), Ordering::Relaxed);
        self.inner
            .timeouts
            .store(Default::default(), Ordering::Relaxed);
        self.inner
            .errors
            .store(Default::default(), Ordering::Relaxed);
        self.inner
            .latency_count
            .store(Default::default(), Ordering::Relaxed);
        self.inner
            .latency_sum
            .store(Default::default(), Ordering::Relaxed);

        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        Temporality::Cumulative
    }
}

impl Metrics {
    pub fn metrics(&self) -> MetricsInfo {
        let latency_count = self.inner.latency_count.load(Ordering::Relaxed);
        let latency_sum = self.inner.latency_sum.load(Ordering::Relaxed);
        let latency = latency_sum as f64 / latency_count as f64;
        MetricsInfo {
            requests: self.inner.requests.load(Ordering::Relaxed),
            processing: self.inner.processing.load(Ordering::Relaxed),
            timeouts: self.inner.timeouts.load(Ordering::Relaxed),
            errors: self.inner.errors.load(Ordering::Relaxed),
            latency,
        }
    }
}
