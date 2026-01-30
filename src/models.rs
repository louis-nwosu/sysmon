use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
pub struct DiskMetrics {
    pub name: String,
    pub file_system: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub is_removable: bool,
    pub is_read_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    pub interface_name: String,
    pub received: u64,
    pub transmitted: u64,
    pub total_received: u64,
    pub total_transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_on_received: u64,
    pub errors_on_transmitted: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_percentage: f32,
    pub user: String,
    pub status: String,
    pub parent_pid: u32,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disks: Vec<DiskMetrics>,
    pub network: NetworkMetrics,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct CpuMetrics {
    pub overall_usage: f32,
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: u64,
    pub load_average: (f32, f32, f32),
}
#[derive(Debug, Clone, Default)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}
