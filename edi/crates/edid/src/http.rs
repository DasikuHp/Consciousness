//! Servidor local (127.0.0.1) del cerebro 3D y de la interacción con EDI. Sin dependencias.
use edi_samn::{Kind, Samn, Src};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct Shared {
    pub state: Mutex<serde_json::Value>,
    pub spikes: Mutex<VecDeque<(u64, Vec<u32>)>>,
    pub samn: Mutex<Samn>,
    pub events: Mutex<VecDeque<String>>,
    pub inbox: Mutex<Vec<String>>,
    pub force_sleep: AtomicBool,
    pub pos: Vec<u8>,
    pub group: Vec<u8>,
    pub region: Vec<u8>,
    pub backbone: Vec<u8>,
    pub activity: Mutex<VecDeque<(u64, Vec<f32>)>>,
    pub face: Mutex<serde_json::Value>,
}

const DASH: &str = include_str!("dashboard.html");
const WIDGET: &str = include_str!("widget.html");

fn respond(s: &mut TcpStream, ctype: &str, body: &[u8]) {
    let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\r\n", body.len());
    let _ = s.write_all(body);
}

fn recall_json(sh: &Shared, word: &str) -> String {
    let mut m = sh.samn.lock().unwrap();
    let seed = m.find(Kind::Word, word).or_else(|| m.find(Kind::Event, word));
    let out = match seed { Some(i) => m.recall(&[(i, 1.0)], None), None => vec![] };
    let items: Vec<_> = out.iter().take(12).map(|(i, a)| {
        let n = &m.nodes[*i as usize];
        serde_json::json!({"kind": format!("{:?}", n.kind), "label": n.label, "a": a, "dreamed": n.src == Src::Dreamed})
    }).collect();
    serde_json::json!({"query": word, "known": seed.is_some(), "recalled": items}).to_string()
}

fn handle(mut s: TcpStream, sh: Arc<Shared>) {
    let mut buf = vec![0u8; 8192];
    let mut n = 0;
    while n < buf.len() {
        match s.read(&mut buf[n..]) { Ok(0) | Err(_) => break, Ok(k) => n += k }
        if let Some(h) = buf[..n].windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..h]).to_lowercase();
            let cl = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
            if n >= h + 4 + cl { break; }
        }
    }
    let req = String::from_utf8_lossy(&buf[..n]).to_string();
    let line = req.lines().next().unwrap_or("");
    let mut it = line.split_whitespace();
    let (method, target) = (it.next().unwrap_or(""), it.next().unwrap_or("/"));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let body = req.split_once("\r\n\r\n").map(|x| x.1).unwrap_or("").to_string();
    let q = |k: &str| query.split('&').find_map(|p| p.strip_prefix(&format!("{k}="))).map(|v| v.to_string());
    match (method, path) {
        ("GET", "/") => respond(&mut s, "text/html; charset=utf-8", DASH.as_bytes()),
        ("GET", "/state") => { let b = sh.state.lock().unwrap().to_string(); respond(&mut s, "application/json", b.as_bytes()) }
        ("GET", "/pos") => respond(&mut s, "application/octet-stream", &sh.pos),
        ("GET", "/group") => respond(&mut s, "application/octet-stream", &sh.group),
        ("GET", "/region") => respond(&mut s, "application/octet-stream", &sh.region),
        ("GET", "/backbone") => respond(&mut s, "application/octet-stream", &sh.backbone),
        ("GET", "/widget") => respond(&mut s, "text/html; charset=utf-8", WIDGET.as_bytes()),
        ("GET", "/face") => { let b = sh.face.lock().unwrap().to_string(); respond(&mut s, "application/json", b.as_bytes()) }
        ("GET", "/activity") => {
            let a = sh.activity.lock().unwrap();
            let v: Vec<_> = a.iter().map(|(t, r)| serde_json::json!({"t": t, "r": r})).collect();
            respond(&mut s, "application/json", serde_json::json!(v).to_string().as_bytes())
        }
        ("GET", p) if p.starts_with("/face/") && p.ends_with(".png") && !p.contains("..") => {
            let dir = std::env::var("EDI_ASSETS").unwrap_or("/usr/share/edi/face".into());
            match std::fs::read(format!("{dir}/{}", &p[6..])) {
                Ok(b) => { let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nCache-Control: max-age=86400\r\n\r\n", b.len()); let _ = s.write_all(&b); }
                Err(_) => { let _ = s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n"); }
            }
        }
        ("GET", "/spikes") => {
            let since: u64 = q("since").and_then(|v| v.parse().ok()).unwrap_or(0);
            let ring = sh.spikes.lock().unwrap();
            let ticks: Vec<_> = ring.iter().filter(|(t, _)| *t > since).map(|(t, v)| serde_json::json!({"t": t, "s": v})).collect();
            let last = ring.back().map(|x| x.0).unwrap_or(0);
            respond(&mut s, "application/json", serde_json::json!({"last": last, "ticks": ticks}).to_string().as_bytes())
        }
        ("GET", "/memory") => {
            let m = sh.samn.lock().unwrap();
            let (nn, ne) = m.stats();
            let top: Vec<_> = m.top(14).into_iter().map(|(_, k, l, a, s)| serde_json::json!({"kind": format!("{k:?}"), "label": l, "a": a, "dreamed": s == Src::Dreamed})).collect();
            respond(&mut s, "application/json", serde_json::json!({"nodes": nn, "edges": ne, "episodes": m.episodes.len(), "top": top}).to_string().as_bytes())
        }
        ("GET", "/events") => {
            let e: Vec<String> = sh.events.lock().unwrap().iter().cloned().collect();
            respond(&mut s, "application/json", serde_json::json!(e).to_string().as_bytes())
        }
        ("GET", "/recall") => {
            let w = q("q").unwrap_or_default().to_lowercase();
            respond(&mut s, "application/json", recall_json(&sh, &w).as_bytes())
        }
        ("POST", "/say") => {
            let text = body.trim().to_string();
            let last = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).last().unwrap_or("").to_lowercase();
            sh.inbox.lock().unwrap().push(text);
            std::thread::sleep(std::time::Duration::from_millis(400));
            respond(&mut s, "application/json", recall_json(&sh, &last).as_bytes())
        }
        ("POST", "/sleep") => { sh.force_sleep.store(true, Ordering::Relaxed); respond(&mut s, "application/json", b"{\"ok\":true}") }
        _ => { let _ = s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n"); }
    }
}

pub fn serve(sh: Arc<Shared>, port: u16) {
    std::thread::spawn(move || {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", port)) {
            for s in l.incoming().flatten() {
                let sh = sh.clone();
                std::thread::spawn(move || handle(s, sh));
            }
        }
    });
}
