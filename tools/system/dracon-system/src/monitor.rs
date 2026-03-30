use crate::contracts::{
    DiskSnapshot, ProcessControlContract, ProcessSnapshot, SystemSnapshot, SystemSnapshotContract,
};
use std::io;
use std::process::Command;
use sysinfo::{Disks, Networks, ProcessesToUpdate, System, Users};

pub struct SystemSnapshotProvider {
    sys: System,
    disks: Disks,
    networks: Networks,
    users: Users,
}

impl SystemSnapshotProvider {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            users: Users::new_with_refreshed_list(),
        }
    }

    fn disk_data(&mut self) -> Vec<DiskSnapshot> {
        let mut disks = Vec::new();

        for disk in self.disks.iter() {
            let mount = disk.mount_point().to_string_lossy();
            let fs_type = disk.file_system().to_string_lossy().to_lowercase();
            let device = disk.name().to_string_lossy().to_string();

            if mount == "/" {
                disks.push(DiskSnapshot {
                    name: mount.to_string(),
                    device,
                    used_space: (disk.total_space() - disk.available_space()) as f64,
                    available_space: disk.available_space() as f64,
                    total_space: disk.total_space() as f64,
                    is_mounted: true,
                });
                continue;
            }

            let is_real_fs = fs_type.contains("ext")
                || fs_type.contains("btrfs")
                || fs_type.contains("xfs")
                || fs_type.contains("zfs")
                || fs_type.contains("vfat")
                || fs_type.contains("fat")
                || fs_type.contains("ntfs")
                || fs_type.contains("exfat")
                || fs_type.contains("fuseblk");

            let is_removable_path = mount.starts_with("/media")
                || mount.starts_with("/mnt")
                || mount.starts_with("/run/media");
            let is_system_path = (mount.starts_with("/boot")
                || mount.starts_with("/nix")
                || mount.starts_with("/run")
                || mount.starts_with("/sys")
                || mount.starts_with("/proc")
                || mount.starts_with("/dev")
                || mount.starts_with("/tmp"))
                && !is_removable_path;

            if is_real_fs
                && (is_removable_path || !is_system_path)
                && disk.total_space() > 100_000_000
            {
                disks.push(DiskSnapshot {
                    name: mount.to_string(),
                    device,
                    used_space: (disk.total_space() - disk.available_space()) as f64,
                    available_space: disk.available_space() as f64,
                    total_space: disk.total_space() as f64,
                    is_mounted: true,
                });
            }
        }

        if let Ok(output) = Command::new("lsblk")
            .arg("-rnbo")
            .arg("NAME,FSTYPE,SIZE,MOUNTPOINT,LABEL")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(' ').collect();
                if parts.len() < 3 {
                    continue;
                }

                let name = parts[0];
                let fstype = parts[1];
                let size_str = parts[2];
                let mountpoint = parts.get(3).copied().unwrap_or("");
                let label = parts.get(4).copied().unwrap_or("");

                if fstype.is_empty() || !mountpoint.is_empty() {
                    continue;
                }
                if fstype == "swap" || fstype.contains("member") {
                    continue;
                }

                if let Ok(size) = size_str.parse::<f64>() {
                    if size <= 100_000_000.0 {
                        continue;
                    }
                    let display_name = if !label.is_empty() {
                        label.to_string()
                    } else {
                        let gb = size / 1_073_741_824.0;
                        if gb >= 1.0 {
                            format!("{gb:.0}G Drive")
                        } else {
                            format!("{:.0}M Drive", size / 1_048_576.0)
                        }
                    };
                    disks.push(DiskSnapshot {
                        name: display_name,
                        device: format!("/dev/{name}"),
                        used_space: 0.0,
                        available_space: size,
                        total_space: size,
                        is_mounted: false,
                    });
                }
            }
        }

        disks
    }
}

impl Default for SystemSnapshotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemSnapshotContract for SystemSnapshotProvider {
    fn capture_snapshot(&mut self) -> io::Result<SystemSnapshot> {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        self.disks.refresh_list();
        self.networks.refresh_list();
        self.users.refresh_list();

        let mut processes = Vec::new();
        for (pid, process) in self.sys.processes() {
            let user = process
                .user_id()
                .and_then(|uid| self.users.iter().find(|u| u.id() == uid))
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| "root".to_string());
            processes.push(ProcessSnapshot {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu: process.cpu_usage(),
                mem: process.memory() as f32 / 1024.0 / 1024.0,
                user,
                status: format!("{:?}", process.status()),
            });
        }
        processes.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(200);

        let mut net_in = 0_u64;
        let mut net_out = 0_u64;
        for (_, data) in &self.networks {
            net_in += data.received();
            net_out += data.transmitted();
        }

        Ok(SystemSnapshot {
            cpu_usage: self.sys.global_cpu_usage(),
            cpu_cores: self.sys.cpus().iter().map(|c| c.cpu_usage()).collect(),
            mem_usage: self.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            total_mem: self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            swap_usage: self.sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0,
            total_swap: self.sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0,
            disks: self.disk_data(),
            processes,
            net_in,
            net_out,
            uptime: System::uptime(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            hostname: System::host_name().unwrap_or_default(),
        })
    }
}

pub struct ProcessController;

impl ProcessControlContract for ProcessController {
    fn kill_process(&self, pid: u32, signal: Option<i32>) -> io::Result<()> {
        let sig = signal.unwrap_or(9).to_string();
        let status = Command::new("kill")
            .args([format!("-{sig}"), pid.to_string()])
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "kill failed for pid {pid} (signal {sig})"
            )))
        }
    }
}
