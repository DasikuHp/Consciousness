//! Sentidos reales del sistema: procesos, puertos, ventana enfocada, presión.
//! Linux: /proc, Niri y PSI del kernel. Windows: API Win32 (ventana, CPU, memoria) + tasklist/netstat.
#[cfg(unix)]
use std::collections::HashSet;
#[cfg(unix)]
use std::process::Command;
#[cfg(windows)]
pub use win::*;

#[cfg(unix)]
pub use linux::*;
#[cfg(unix)]
mod linux {
use super::*;

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
}

#[cfg(windows)]
mod win {
    use std::collections::HashSet;
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use std::sync::Mutex;
    const NO_WINDOW: u32 = 0x0800_0000;

    #[repr(C)] #[derive(Default, Clone, Copy)] struct FileTime { lo: u32, hi: u32 }
    #[repr(C)] struct MemStatus { len: u32, load: u32, rest: [u64; 7] }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
        fn GlobalMemoryStatusEx(m: *mut MemStatus) -> i32;
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> isize;
        fn QueryFullProcessImageNameW(h: isize, flags: u32, buf: *mut u16, len: *mut u32) -> i32;
        fn CloseHandle(h: isize) -> i32;
    }
    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> isize;
        fn GetWindowThreadProcessId(h: isize, pid: *mut u32) -> u32;
    }

    fn run(cmd: &str, args: &[&str]) -> String {
        Command::new(cmd).args(args).creation_flags(NO_WINDOW).output().map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
    }

    pub fn procs() -> HashSet<String> {
        run("tasklist", &["/fo", "csv", "/nh"]).lines()
            .filter_map(|l| l.split("\",\"").next()).map(|n| n.trim_matches('"').trim_end_matches(".exe").to_string())
            .filter(|n| !n.is_empty()).collect()
    }

    pub fn ports() -> HashSet<u16> {
        run("netstat", &["-ano", "-p", "tcp"]).lines()
            .filter(|l| l.contains("LISTENING"))
            .filter_map(|l| l.split_whitespace().nth(1)?.rsplit(':').next()?.parse().ok()).collect()
    }

    /// Programa de la ventana en primer plano (p. ej. "firefox", "Code").
    pub fn focused() -> Option<String> {
        unsafe {
            let h = GetForegroundWindow();
            if h == 0 { return None; }
            let mut pid = 0u32;
            GetWindowThreadProcessId(h, &mut pid);
            let p = OpenProcess(0x1000, 0, pid); // PROCESS_QUERY_LIMITED_INFORMATION
            if p == 0 { return None; }
            let mut buf = [0u16; 512];
            let mut n = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(p, 0, buf.as_mut_ptr(), &mut n);
            CloseHandle(p);
            if ok == 0 { return None; }
            let path = String::from_utf16_lossy(&buf[..n as usize]);
            Some(path.rsplit('\\').next()?.trim_end_matches(".exe").to_string())
        }
    }

    static LAST: Mutex<Option<(u64, u64)>> = Mutex::new(None);
    fn ft(f: FileTime) -> u64 { (f.hi as u64) << 32 | f.lo as u64 }

    /// Windows no tiene PSI: aproximación a "presión" (0-100) desde la ocupación de CPU y memoria.
    pub fn psi(kind: &str) -> f64 {
        unsafe {
            if kind == "memory" {
                let mut m = MemStatus { len: std::mem::size_of::<MemStatus>() as u32, load: 0, rest: [0; 7] };
                if GlobalMemoryStatusEx(&mut m) == 0 { return 0.0; }
                return ((m.load as f64 - 80.0) * 5.0).clamp(0.0, 100.0);
            }
            let (mut i, mut k, mut u) = (FileTime::default(), FileTime::default(), FileTime::default());
            if GetSystemTimes(&mut i, &mut k, &mut u) == 0 { return 0.0; }
            let (idle, total) = (ft(i), ft(k) + ft(u)); // kernel incluye idle
            let mut last = LAST.lock().unwrap();
            let busy = match *last { Some((li, lt)) if total > lt => 1.0 - (idle - li) as f64 / (total - lt) as f64, _ => 0.0 };
            *last = Some((idle, total));
            ((busy * 100.0 - 50.0) * 2.0).clamp(0.0, 100.0)
        }
    }
}
