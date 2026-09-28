//! Sentidos reales del sistema (Linux): procesos, puertos, ventana enfocada (Niri), presión.
use std::collections::HashSet;
use std::process::Command;

pub fn procs() -> HashSet<String> {
    let mut s = HashSet::new();
    if let Ok(rd) = std::fs::read_dir("/proc") {
        for e in rd.flatten() {
            let n = e.file_name();
            let n = n.to_string_lossy();
            if n.bytes().all(|b| b.is_ascii_digit()) {
                if let Ok(c) = std::fs::read_to_string(format!("/proc/{n}/comm")) {
                    let c = c.trim().to_string();
                    if !c.contains('/') && !c.is_empty() { s.insert(c); }
                }
            }
        }
    }
    s
}

pub fn ports() -> HashSet<u16> {
    let mut s = HashSet::new();
    for f in ["/proc/net/tcp", "/proc/net/tcp6"] {
        if let Ok(t) = std::fs::read_to_string(f) {
            for l in t.lines().skip(1) {
                let c: Vec<&str> = l.split_whitespace().collect();
                if c.len() > 3 && c[3] == "0A" {
                    if let Some(p) = c[1].rsplit(':').next().and_then(|h| u16::from_str_radix(h, 16).ok()) { s.insert(p); }
                }
            }
        }
    }
    s
}

pub fn focused() -> Option<String> {
    let o = Command::new("niri").args(["msg", "-j", "focused-window"]).output().ok()?;
    if !o.status.success() { return None; }
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).ok()?;
    v.get("app_id")?.as_str().map(|s| s.to_string())
}

pub fn psi(kind: &str) -> f64 {
    std::fs::read_to_string(format!("/proc/pressure/{kind}")).ok()
        .and_then(|s| s.lines().next().and_then(|l| l.split_whitespace()
            .find_map(|t| t.strip_prefix("avg10=")).and_then(|x| x.parse().ok())))
        .unwrap_or(0.0)
}
