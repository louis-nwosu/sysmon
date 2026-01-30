use crate::models::*;
use chrono::Utc;
use sysinfo::{Disks, Networks, System};

pub struct SystemCollector {
    system: System,
    networks: Networks,
    disks: Disks,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn collect(&mut self) -> Result<SystemMetrics, anyhow::Error> {
        self.system.refresh_all();
        self.networks.refresh_list();
        self.networks.refresh();
        self.disks.refresh_list();

        let cpu = self.collect_cpu_metrics();
        let memory = self.collect_memory_metrics();

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            cpu,
            memory,
            disks: self.collect_disk_metrics(),
            network: self.collect_network_metrics(),
            processes: self.collect_process_metrics(),
        })
    }

    fn collect_cpu_metrics(&self) -> CpuMetrics {
        let global_cpu = self.system.global_cpu_info();

        CpuMetrics {
            overall_usage: global_cpu.cpu_usage(),
            per_core_usage: self
                .system
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect(),
            frequency_mhz: global_cpu.frequency(),
            load_average: (
                System::load_average().one as f32,
                System::load_average().five as f32,
                System::load_average().fifteen as f32,
            ),
        }
    }

    fn collect_memory_metrics(&self) -> MemoryMetrics {
        MemoryMetrics {
            total_bytes: self.system.total_memory(),
            used_bytes: self.system.used_memory(),
            available_bytes: self.system.available_memory(),
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        }
    }

    fn collect_disk_metrics(&self) -> Vec<DiskMetrics> {
        self.disks
            .list()
            .iter()
            .map(|disk| DiskMetrics {
                name: disk.name().to_string_lossy().into_owned(),
                file_system: disk.file_system().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                is_removable: disk.is_removable(),
                is_read_only: false,
            })
            .collect()
    }

    fn collect_network_metrics(&self) -> NetworkMetrics {
        let (rx, tx, total_rx, total_tx, packets_rx, packets_tx, errors_rx, errors_tx) = self
            .networks
            .iter()
            .fold((0, 0, 0, 0, 0, 0, 0, 0), |acc, (_name, data)| {
                (
                    acc.0 + data.received(),
                    acc.1 + data.transmitted(),
                    acc.2 + data.total_received(),
                    acc.3 + data.total_transmitted(),
                    acc.4 + data.packets_received(),
                    acc.5 + data.packets_transmitted(),
                    acc.6 + data.errors_on_received(),
                    acc.7 + data.errors_on_transmitted(),
                )
            });

        NetworkMetrics {
            interface_name: "Aggregated".to_string(),
            received: rx,
            transmitted: tx,
            total_received: total_rx,
            total_transmitted: total_tx,
            packets_received: packets_rx,
            packets_transmitted: packets_tx,
            errors_on_received: errors_rx,
            errors_on_transmitted: errors_tx,
        }
    }

    fn collect_process_metrics(&self) -> Vec<ProcessInfo> {
        self.system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                command: process.cmd().join(" "),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                memory_percentage: 0.0,
                user: process
                    .user_id()
                    .map(|uid| uid.to_string())
                    .unwrap_or_default(),
                status: process.status().to_string(),
                parent_pid: process.parent().map(|p| p.as_u32()).unwrap_or(0),
                start_time: chrono::DateTime::from_timestamp(process.start_time() as i64, 0)
                    .unwrap_or_default()
                    .into(),
            })
            .collect()
    }
}
