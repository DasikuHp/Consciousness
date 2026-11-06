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

/// Qué programa escucha en ese puerto (solo se ven los procesos propios sin ser root).
pub fn port_owner(port: u16) -> Option<String> {
    let mut inode = None;
    for f in ["/proc/net/tcp", "/proc/net/tcp6"] {
        let t = std::fs::read_to_string(f).ok()?;
        for l in t.lines().skip(1) {
            let c: Vec<&str> = l.split_whitespace().collect();
            if c.len() > 9 && c[3] == "0A" && c[1].rsplit(':').next().and_then(|h| u16::from_str_radix(h, 16).ok()) == Some(port) { inode = Some(c[9].to_string()); break; }
        }
        if inode.is_some() { break; }
    }
    let want = format!("socket:[{}]", inode?);
    for e in std::fs::read_dir("/proc").ok()?.flatten() {
        let pid = e.file_name().to_string_lossy().into_owned();
        if !pid.bytes().all(|b| b.is_ascii_digit()) { continue; }
        let Ok(fds) = std::fs::read_dir(format!("/proc/{pid}/fd")) else { continue };
        for fd in fds.flatten() {
            if std::fs::read_link(fd.path()).map(|l| l.to_string_lossy() == want).unwrap_or(false) {
                return std::fs::read_to_string(format!("/proc/{pid}/comm")).ok().map(|s| s.trim().to_string());
            }
        }
    }
    None
}

/// Diferencia de la hora local con UTC, en segundos (para «a las 18:30» y el diario).
pub fn local_offset() -> i64 {
    let o = Command::new("date").arg("+%z").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    if o.len() != 5 { return 0; }
    let sign = if o.starts_with('-') { -1 } else { 1 };
    let (h, m): (i64, i64) = (o[1..3].parse().unwrap_or(0), o[3..5].parse().unwrap_or(0));
    sign * (h * 3600 + m * 60)
}

/// Aviso de escritorio (notify-send). EDI_NOTIFY=0 lo apaga.
pub fn notify(title: &str, body: &str, icon: Option<&str>) {
    if std::env::var("EDI_NOTIFY").as_deref() == Ok("0") { return; }
    let mut c = Command::new("notify-send");
    c.args(["-a", "EDI"]);
    if let Some(i) = icon { c.args(["-i", i]); }
    let _ = c.args([title, body]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
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

    /// Qué programa escucha en ese puerto (netstat -ano → PID → tasklist).
    pub fn port_owner(port: u16) -> Option<String> {
        let pid = run("netstat", &["-ano", "-p", "tcp"]).lines().filter(|l| l.contains("LISTENING")).find_map(|l| {
            let c: Vec<&str> = l.split_whitespace().collect();
            (c.len() >= 5 && c[1].rsplit(':').next()?.parse::<u16>().ok()? == port).then(|| c[4].to_string())
        })?;
        let out = run("tasklist", &["/fi", &format!("PID eq {pid}"), "/fo", "csv", "/nh"]);
        out.lines().next()?.split("\",\"").next().map(|n| n.trim_matches('"').trim_end_matches(".exe").to_string()).filter(|n| !n.is_empty() && !n.starts_with("INFO"))
    }

    #[repr(C)] struct SysTime([u16; 8]);
    #[repr(C)] struct TzInfo { bias: i32, sn: [u16; 32], sd: SysTime, sb: i32, dn: [u16; 32], dd: SysTime, db: i32 }
    #[link(name = "kernel32")]
    extern "system" { fn GetTimeZoneInformation(tz: *mut TzInfo) -> u32; }

    /// Diferencia de la hora local con UTC, en segundos.
    pub fn local_offset() -> i64 {
        unsafe {
            let mut tz: TzInfo = std::mem::zeroed();
            let r = GetTimeZoneInformation(&mut tz);
            let bias = tz.bias + if r == 2 { tz.db } else if r == 1 { tz.sb } else { 0 };
            -(bias as i64) * 60
        }
    }

    /// Aviso de Windows (toast) con PowerShell 5.1, sin módulos; el texto va por variables de entorno.
    pub fn notify(title: &str, body: &str, _icon: Option<&str>) {
        if std::env::var("EDI_NOTIFY").as_deref() == Ok("0") { return; }
        let ps = "[Windows.UI.Notifications.ToastNotificationManager,Windows.UI.Notifications,ContentType=WindowsRuntime]>$null;\
$x=[Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02);\
$t=$x.GetElementsByTagName('text');$t[0].AppendChild($x.CreateTextNode($env:EDI_T))>$null;$t[1].AppendChild($x.CreateTextNode($env:EDI_B))>$null;\
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe').Show([Windows.UI.Notifications.ToastNotification]::new($x))";
        let _ = Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", ps]).env("EDI_T", title).env("EDI_B", body).creation_flags(NO_WINDOW).spawn();
    }
}
