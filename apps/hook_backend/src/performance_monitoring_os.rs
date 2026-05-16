use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use storage::performance_monitoring::SystemMetricsSnapshot;
use sysinfo::{CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use types::performance_monitoring::{HostResourceMetrics, MetricSupportStatus, NetworkConnectionMetrics};

use crate::{performance_monitoring_disk::DiskSpaceCollector, performance_monitoring_tcp::tcp_snapshot};

const MIN_RATE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, thiserror::Error)]
pub enum PerformanceOsCollectorError {
    #[error("failed to lock performance OS collector")]
    LockPoisoned,
    #[error("performance OS collector task failed: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
    #[error("failed to locate backend working directory for disk metrics: {0}")]
    WorkingDirectory(#[from] std::io::Error),
    #[error("current process pid is unsupported by sysinfo: {0}")]
    CurrentPidUnsupported(&'static str),
    #[error("current process is unsupported by sysinfo: {0}")]
    CurrentProcessUnsupported(&'static str),
}

pub struct PerformanceOsCollector {
    state: Mutex<CollectorState>,
}

impl PerformanceOsCollector {
    pub fn new() -> Result<Self, PerformanceOsCollectorError> {
        let disk_space_collector = DiskSpaceCollector::for_current_dir()?;
        Ok(Self {
            state: Mutex::new(CollectorState::new(disk_space_collector)?),
        })
    }

    pub async fn snapshot(self: std::sync::Arc<Self>) -> Result<SystemMetricsSnapshot, PerformanceOsCollectorError> {
        tokio::task::spawn_blocking(move || self.collect_snapshot()).await?
    }

    fn collect_snapshot(&self) -> Result<SystemMetricsSnapshot, PerformanceOsCollectorError> {
        let mut state = self.state.lock().map_err(|_| PerformanceOsCollectorError::LockPoisoned)?;
        Ok(SystemMetricsSnapshot {
            network: state.network(),
            host: state.host()?,
        })
    }
}

struct CollectorState {
    system: System,
    networks: Networks,
    disks: Disks,
    disk_space_collector: DiskSpaceCollector,
    current_pid: sysinfo::Pid,
    last_network_at: Instant,
    last_disk_at: Instant,
    last_disk_write_rate: f64,
}

impl CollectorState {
    fn new(disk_space_collector: DiskSpaceCollector) -> Result<Self, PerformanceOsCollectorError> {
        let current_pid = sysinfo::get_current_pid().map_err(PerformanceOsCollectorError::CurrentPidUnsupported)?;
        let mut state = Self {
            system: System::new_with_specifics(refresh_kind()),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything()),
            disk_space_collector,
            current_pid,
            last_network_at: Instant::now(),
            last_disk_at: Instant::now(),
            last_disk_write_rate: 0.0,
        };
        state.refresh_process();
        Ok(state)
    }

    fn host(&mut self) -> Result<HostResourceMetrics, PerformanceOsCollectorError> {
        self.refresh_system();
        let load = System::load_average();
        let process = current_process_metrics(&self.system, self.current_pid)?;
        let disk_space = self.disk_space_collector.snapshot(&self.disks);
        let disk_read_rate = self.disk_read_rate();
        Ok(HostResourceMetrics {
            cpu_usage_percent: Some(self.system.global_cpu_usage() as f64),
            load_average_1m: Some(load.one),
            load_average_5m: Some(load.five),
            load_average_15m: Some(load.fifteen),
            memory_rss_bytes: process.memory_rss_bytes,
            memory_usage_bytes: to_i64(self.system.used_memory()),
            disk_total_bytes: disk_space.map(|value| value.total_bytes).and_then(to_i64),
            disk_available_bytes: disk_space.map(|value| value.available_bytes).and_then(to_i64),
            disk_read_bytes_per_second: Some(disk_read_rate),
            disk_write_bytes_per_second: Some(self.disk_write_rate()),
            file_descriptors: process.file_descriptors,
            threads: process.threads,
            processes: to_i64(self.system.processes().len()),
            status: MetricSupportStatus::Ready,
        })
    }

    fn network(&mut self) -> NetworkConnectionMetrics {
        let seconds = elapsed_seconds(self.last_network_at.elapsed());
        self.networks.refresh(true);
        self.last_network_at = Instant::now();
        let traffic = network_traffic(&self.networks);
        let tcp = tcp_snapshot().map_err(log_tcp_error).ok();
        NetworkConnectionMetrics {
            inbound_bytes: saturating_i64(traffic.total_received),
            outbound_bytes: saturating_i64(traffic.total_transmitted),
            inbound_bandwidth_bytes_per_second: rate(traffic.received, seconds),
            outbound_bandwidth_bytes_per_second: rate(traffic.transmitted, seconds),
            current_connections: tcp.as_ref().map(|value| value.current_connections),
            new_connections_per_second: tcp.as_ref().map(|value| rate(value.new_connections as u64, seconds)),
            tcp_total: tcp.as_ref().map(|value| value.tcp_total),
            tcp_time_wait: tcp.as_ref().map(|value| value.time_wait),
            tcp_established: tcp.as_ref().map(|value| value.established),
            tcp_close_wait: tcp.as_ref().map(|value| value.close_wait),
            retransmits: None,
            packet_loss: to_i64(traffic.total_errors),
            status: MetricSupportStatus::Ready,
        }
    }

    fn refresh_system(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        self.system
            .refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing().without_tasks());
        self.refresh_process();
        self.disks.refresh_specifics(true, DiskRefreshKind::nothing().with_storage());
    }

    fn refresh_process(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.current_pid]),
            true,
            ProcessRefreshKind::nothing()
                .with_memory()
                .with_disk_usage()
                .with_tasks()
                .with_exe(UpdateKind::OnlyIfNotSet),
        );
    }

    fn disk_read_rate(&mut self) -> f64 {
        let seconds = elapsed_seconds(self.last_disk_at.elapsed());
        let mut read_bytes = 0;
        let mut written_bytes = 0;
        for disk in self.disks.list_mut() {
            disk.refresh_specifics(DiskRefreshKind::nothing().with_io_usage());
            let usage = disk.usage();
            read_bytes += usage.read_bytes;
            written_bytes += usage.written_bytes;
        }
        self.last_disk_at = Instant::now();
        self.last_disk_write_rate = rate(written_bytes, seconds);
        rate(read_bytes, seconds)
    }

    fn disk_write_rate(&self) -> f64 {
        self.last_disk_write_rate
    }
}

fn refresh_kind() -> RefreshKind {
    RefreshKind::nothing()
        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
        .with_memory(MemoryRefreshKind::nothing().with_ram())
}

fn network_traffic(networks: &Networks) -> NetworkTraffic {
    networks.iter().fold(NetworkTraffic::default(), |current, (_, data)| NetworkTraffic {
        received: current.received + data.received(),
        transmitted: current.transmitted + data.transmitted(),
        total_received: current.total_received + data.total_received(),
        total_transmitted: current.total_transmitted + data.total_transmitted(),
        total_errors: current.total_errors + data.total_errors_on_received() + data.total_errors_on_transmitted(),
    })
}

fn log_tcp_error(error: netstat2::error::Error) -> netstat2::error::Error {
    hook_tracing::warn_with_fields!("performance TCP socket snapshot unavailable", error = error);
    error
}

fn current_process_metrics(system: &System, pid: sysinfo::Pid) -> Result<CurrentProcessMetrics, PerformanceOsCollectorError> {
    let process = system
        .process(pid)
        .ok_or(PerformanceOsCollectorError::CurrentProcessUnsupported("process not found"))?;
    Ok(CurrentProcessMetrics {
        memory_rss_bytes: to_i64(process.memory()),
        file_descriptors: process.open_files().and_then(to_i64),
        threads: process.tasks().map(|tasks| tasks.len()).and_then(to_i64),
    })
}

fn elapsed_seconds(elapsed: Duration) -> f64 {
    elapsed.max(MIN_RATE_INTERVAL).as_secs_f64()
}

fn rate(count: u64, seconds: f64) -> f64 {
    count as f64 / seconds
}

fn saturating_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn to_i64<T>(value: T) -> Option<i64>
where
    i64: TryFrom<T>,
{
    i64::try_from(value).ok()
}

#[derive(Default)]
struct NetworkTraffic {
    received: u64,
    transmitted: u64,
    total_received: u64,
    total_transmitted: u64,
    total_errors: u64,
}

struct CurrentProcessMetrics {
    memory_rss_bytes: Option<i64>,
    file_descriptors: Option<i64>,
    threads: Option<i64>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{elapsed_seconds, saturating_i64};
    use types::performance_monitoring::MetricSupportStatus;

    #[test]
    fn elapsed_seconds_has_minimum_for_rate_stability() {
        assert_eq!(elapsed_seconds(Duration::ZERO), 0.1);
    }

    #[test]
    fn u64_to_i64_conversion_saturates_at_i64_max() {
        assert_eq!(saturating_i64(u64::MAX), i64::MAX);
    }

    #[tokio::test]
    async fn collector_snapshot_reports_ready_metrics() {
        let snapshot = std::sync::Arc::new(super::PerformanceOsCollector::new().unwrap()).snapshot().await.unwrap();

        assert_eq!(snapshot.host.status, MetricSupportStatus::Ready);
        assert_eq!(snapshot.network.status, MetricSupportStatus::Ready);
        assert!(snapshot.host.cpu_usage_percent.is_some());
        assert!(snapshot.host.memory_usage_bytes.is_some_and(|value| value > 0));
        assert!(snapshot.host.disk_total_bytes.is_some_and(|value| value > 0));
        assert!(snapshot.network.inbound_bytes >= 0);
        assert!(snapshot.network.outbound_bytes >= 0);
    }
}
