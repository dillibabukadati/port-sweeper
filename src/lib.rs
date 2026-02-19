//! Core library for Port Sweeper: list processes by port, kill by port, parse port specs.

use listeners::{Protocol, get_all, get_process_by_port};
use std::collections::BTreeSet;
use std::net::IpAddr;
use sysinfo::{Pid, System};

/// A row for display: one listening port with process info.
#[derive(Debug, Clone)]
pub struct PortEntry {
    pub port: u16,
    pub process_name: String,
    pub pid: u32,
    pub status: String,
}

/// Result of killing a single port.
#[derive(Debug, Clone)]
pub struct KillResult {
    pub port: u16,
    pub success: bool,
    pub message: String,
}

/// Parse a port spec string into a list of port numbers.
/// Accepts: "3000", "3000,8000", "9000-9010", "3000,8000,9000-9010"
/// Ports must be in 1..=65535.
pub fn parse_port_spec(s: &str) -> Result<Vec<u16>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let mut ports = BTreeSet::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            let start: u16 = a.trim().parse().map_err(|_| format!("invalid port: {}", a))?;
            let end: u16 = b.trim().parse().map_err(|_| format!("invalid port: {}", b))?;
            if start == 0 || end == 0 {
                return Err("ports must be between 1 and 65535".to_string());
            }
            if start > end {
                return Err(format!("invalid range: {} > {}", start, end));
            }
            for p in start..=end {
                if p == 0 {
                    return Err("port 0 is not valid".to_string());
                }
                ports.insert(p);
            }
        } else {
            let p: u16 = part.parse().map_err(|_| format!("invalid port: {}", part))?;
            if p == 0 {
                return Err("port 0 is not valid".to_string());
            }
            ports.insert(p);
        }
    }
    Ok(ports.into_iter().collect())
}

/// List all listening ports with process info. Deduped by (port, pid).
/// Filters to TCP listeners (and optionally UDP); only includes bindings that have a port.
pub fn list_ports() -> Result<Vec<PortEntry>, String> {
    let listeners = get_all().map_err(|e| e.to_string())?;
    let mut seen = BTreeSet::<(u16, u32)>::new();
    let mut entries = Vec::new();
    for l in listeners {
        if l.protocol != Protocol::TCP {
            continue;
        }
        let port = l.socket.port();
        if port == 0 {
            continue;
        }
        // Only include "listening" bindings: 0.0.0.0 or :: or similar
        let ip = l.socket.ip();
        if !matches!(ip, IpAddr::V4(a) if a.is_unspecified())
            && !matches!(ip, IpAddr::V6(a) if a.is_unspecified())
        {
            continue;
        }
        if seen.insert((port, l.process.pid)) {
            entries.push(PortEntry {
                port,
                process_name: l.process.name.clone(),
                pid: l.process.pid,
                status: "Running".to_string(),
            });
        }
    }
    entries.sort_by_key(|e| (e.port, e.pid));
    Ok(entries)
}

/// Kill the process listening on the given port. Returns (success, message).
pub fn kill_port(port: u16) -> (bool, String) {
    let process = match get_process_by_port(port, Protocol::TCP) {
        Ok(p) => p,
        Err(e) => return (false, format!("Port {}: {}", port, e)),
    };
    let mut sys = System::new_all();
    sys.refresh_all();
    let pid = Pid::from_u32(process.pid);
    let proc_ref = match sys.process(pid) {
        Some(p) => p,
        None => return (false, format!("Port {}: process {} not found", port, process.pid)),
    };
    if proc_ref.kill() {
        (true, format!("Port {} terminated successfully!", port))
    } else {
        (false, format!("Port {}: failed to kill process (try running with elevated permissions)", port))
    }
}

/// Kill processes on all ports in the spec. Returns one result per port.
pub fn kill_ports(ports: &[u16]) -> Vec<KillResult> {
    ports
        .iter()
        .map(|&port| {
            let (success, message) = kill_port(port);
            KillResult { port, success, message }
        })
        .collect()
}

pub mod gui;

#[cfg(target_os = "macos")]
pub mod installer_macos;
