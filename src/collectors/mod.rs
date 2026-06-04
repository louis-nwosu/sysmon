use std::collections::HashMap;
use std::time::Instant;

use crate::models::*;
use chrono::Utc;
use sysinfo::{Components, Disks, Networks, Signal, System, Users};

pub struct SystemCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    users: Users,
    prev_net: HashMap<String, (u64, u64, Instant)>,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            users: Users::new_with_refreshed_list(),
            prev_net: HashMap::new(),
        }
    }

    pub fn collect_sorted(&mut self, order: SortOrder) -> Result<SystemMetrics, anyhow::Error> {
        self.system.refresh_all();
        self.networks.refresh_list();
        self.networks.refresh();
        self.disks.refresh_list();
        self.components.refresh();
        self.users.refresh_list();

        let info = self.collect_system_info();
        let cpu = self.collect_cpu_metrics();
        let memory = self.collect_memory_metrics();

        Ok(SystemMetrics {
            info,
            timestamp: Utc::now(),
            cpu,
            memory,
            disks: self.collect_disk_metrics(),
            network: self.collect_network_aggregated(),
            network_interfaces: self.collect_network_per_interface(),
            temperatures: self.collect_temperatures(),
            processes: self.collect_process_metrics(order),
        })
    }

    pub fn kill_process(&self, pid: u32) -> bool {
        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(pid)) {
            process.kill_with(Signal::Term) == Some(true)
        } else {
            false
        }
    }

    fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::long_os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            cpu_arch: System::cpu_arch().unwrap_or_default(),
            uptime: System::uptime(),
            boot_time: System::boot_time(),
        }
    }

    fn collect_cpu_metrics(&self) -> CpuMetrics {
        let global_cpu = self.system.global_cpu_info();
        let cpus = self.system.cpus();

        CpuMetrics {
            overall_usage: global_cpu.cpu_usage(),
            per_core_usage: cpus.iter().map(|cpu| cpu.cpu_usage()).collect(),
            frequency_mhz: global_cpu.frequency(),
            brand: cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
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
                disk_kind: format!("{:?}", disk.kind()),
            })
            .collect()
    }

    fn collect_temperatures(&self) -> Vec<TemperatureMetric> {
        self.components
            .iter()
            .map(|c| TemperatureMetric {
                label: c.label().to_string(),
                temperature: c.temperature(),
                max: c.max(),
                critical: c.critical(),
            })
            .collect()
    }

    fn calc_speed(
        prev: &mut HashMap<String, (u64, u64, Instant)>,
        name: &str,
        rx: u64,
        tx: u64,
    ) -> (f64, f64) {
        let now = Instant::now();
        let (rx_ps, tx_ps) = if let Some(&(prx, ptx, pt)) = prev.get(name) {
            let dt = (now - pt).as_secs_f64().max(0.001);
            (
                if prx > 0 { rx.saturating_sub(prx) as f64 / dt } else { 0.0 },
                if ptx > 0 { tx.saturating_sub(ptx) as f64 / dt } else { 0.0 },
            )
        } else {
            (0.0, 0.0)
        };
        prev.insert(name.to_string(), (rx, tx, now));
        (rx_ps, tx_ps)
    }

    fn collect_network_aggregated(&mut self) -> NetworkMetrics {
        let mut rx = 0u64;
        let mut tx = 0u64;
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        let mut packets_rx = 0u64;
        let mut packets_tx = 0u64;
        let mut errors_rx = 0u64;
        let mut errors_tx = 0u64;

        for (_name, data) in self.networks.iter() {
            rx += data.received();
            tx += data.transmitted();
            total_rx += data.total_received();
            total_tx += data.total_transmitted();
            packets_rx += data.packets_received();
            packets_tx += data.packets_transmitted();
            errors_rx += data.errors_on_received();
            errors_tx += data.errors_on_transmitted();
        }

        let (rx_ps, tx_ps) = Self::calc_speed(&mut self.prev_net, "__aggregated__", rx, tx);

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
            rx_per_sec: rx_ps,
            tx_per_sec: tx_ps,
            mac_address: String::new(),
        }
    }

    fn collect_network_per_interface(&mut self) -> Vec<NetworkMetrics> {
        self.networks
            .iter()
            .map(|(name, data)| {
                let rx = data.received();
                let tx = data.transmitted();
                let (rx_ps, tx_ps) = Self::calc_speed(&mut self.prev_net, name, rx, tx);
                NetworkMetrics {
                    interface_name: name.to_string(),
                    received: rx,
                    transmitted: tx,
                    total_received: data.total_received(),
                    total_transmitted: data.total_transmitted(),
                    packets_received: data.packets_received(),
                    packets_transmitted: data.packets_transmitted(),
                    errors_on_received: data.errors_on_received(),
                    errors_on_transmitted: data.errors_on_transmitted(),
                    rx_per_sec: rx_ps,
                    tx_per_sec: tx_ps,
                    mac_address: format!("{}", data.mac_address()),
                }
            })
            .collect()
    }

    fn resolve_user(&self, uid: Option<&sysinfo::Uid>) -> String {
        match uid {
            Some(uid) => self
                .users
                .get_user_by_id(uid)
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| uid.to_string()),
            None => "?".to_string(),
        }
    }

    fn collect_process_metrics(&self, order: SortOrder) -> Vec<ProcessInfo> {
        let total_mem = self.system.total_memory();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let mem = process.memory();
                let mem_pct = if total_mem > 0 {
                    (mem as f64 * 100.0 / total_mem as f64) as f32
                } else {
                    0.0
                };
                let start = process.start_time() as i64;
                let runtime = if start > 0 && now > start {
                    (now - start) as u64
                } else {
                    0
                };
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    command: process.cmd().join(" "),
                    cpu_usage: process.cpu_usage(),
                    memory_usage: mem,
                    memory_percentage: mem_pct,
                    virtual_memory: process.virtual_memory(),
                    user: self.resolve_user(process.user_id()),
                    status: process.status().to_string(),
                    parent_pid: process.parent().map(|p| p.as_u32()).unwrap_or(0),
                    start_time: chrono::DateTime::from_timestamp(start, 0)
                        .unwrap_or_default()
                        .into(),
                    run_time: runtime,
                }
            })
            .collect();

        match order {
            SortOrder::Cpu => {
                processes.sort_by(|a, b| {
                    b.cpu_usage
                        .partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortOrder::Memory => {
                processes.sort_by(|a, b| {
                    b.memory_percentage
                        .partial_cmp(&a.memory_percentage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortOrder::Pid => {
                processes.sort_by_key(|p| p.pid);
            }
        }

        processes
    }
}
