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
    /// frases tuyas pendientes de oír: (turno, texto)
    pub inbox: Mutex<Vec<(u64, String)>>,
    pub chat: Mutex<crate::talk::ChatLog>,
    pub fb: Mutex<Vec<crate::talk::Feedback>>,
    pub force_sleep: AtomicBool,
    pub pos: Vec<u8>,
    pub group: Vec<u8>,
    pub region: Vec<u8>,
    pub backbone: Vec<u8>,
    pub activity: Mutex<VecDeque<(u64, Vec<f32>)>>,
    pub face: Mutex<serde_json::Value>,
    pub learn: Mutex<serde_json::Value>,
    /// (longitud, vocabulario, historia reciente de conceptos (ROSA), valencias medidas)
    pub seq: Mutex<(usize, Vec<String>, Vec<u32>, Vec<(String, f32)>)>,
    /// eventos de sentidos externos (/sense: comandos de la terminal, notas…): (etiqueta, valencia)
    pub sensed: Mutex<Vec<(String, f32)>>,
    /// texto que oye sin contestar (las notas): entra al oído y a la memoria
    pub inbox_quiet: Mutex<Vec<String>>,
    pub reminders: Mutex<Vec<crate::uso::Reminder>>,
    pub state_dir: std::path::PathBuf,
    /// hora local − UTC (s)
    pub tz_offset: i64,
}

const DASH: &str = include_str!("dashboard.html");
const WIDGET: &str = include_str!("widget.html");
const CHATJS: &str = include_str!("chat.js");

/// Decodifica %XX y '+' de una query (UTF-8).
fn pct(v: &str) -> String {
    let b = v.as_bytes();
    let (mut out, mut i) = (Vec::with_capacity(b.len()), 0);
    while i < b.len() {
        let hex = if b[i] == b'%' && i + 2 < b.len() { std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) } else { None };
        match (b[i], hex) {
            (b'+', _) => out.push(b' '),
            (_, Some(x)) => { out.push(x); i += 2; }
            (c, None) => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn respond(s: &mut TcpStream, ctype: &str, body: &[u8]) {
    let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\r\n", body.len());
    let _ = s.write_all(body);
}

pub fn recall_json(sh: &Shared, word: &str) -> String {
    let mut m = sh.samn.lock().unwrap();
    let seed = m.find(Kind::Word, word).or_else(|| m.find(Kind::Event, word));
    let out = match seed { Some(i) => m.recall(&[(i, 1.0)], None), None => vec![] };
    let items: Vec<_> = out.iter().take(12).map(|(i, a)| {
        let n = &m.nodes[*i as usize];
        serde_json::json!({"kind": format!("{:?}", n.kind), "label": n.label, "a": a, "dreamed": n.src == Src::Dreamed})
    }).collect();
    drop(m);
    // ROSA: lo que vino justo después la última vez (recuerdo episódico exacto, en orden)
    let sq = sh.seq.lock().unwrap();
    let after: Vec<String> = sq.1.iter().position(|l| l == &format!("OYE:{word}") || l == word)
        .and_then(|t| sq.2[..sq.2.len().saturating_sub(1)].iter().rposition(|&x| x as usize == t))
        .map(|j| sq.2[j + 1..(j + 6).min(sq.2.len())].iter().map(|&x| sq.1[x as usize].clone()).collect()).unwrap_or_default();
    let val = sq.3.iter().find(|(l, _)| l == &format!("OYE:{word}") || l == word).map(|x| x.1);
    serde_json::json!({"query": word, "known": seed.is_some(), "recalled": items, "despues": after, "valencia": val}).to_string()
}

fn handle(mut s: TcpStream, sh: Arc<Shared>) {
    let mut buf = vec![0u8; 65536];
    let mut n = 0;
    while n < buf.len() {
        match s.read(&mut buf[n..]) { Ok(0) | Err(_) => break, Ok(k) => n += k }
        if let Some(h) = buf[..n].windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..h]).to_lowercase();
            let cl = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
            if n >= h + 4 + cl { break; }
        }
    }
    let hend = buf[..n].windows(4).position(|w| w == b"\r\n\r\n").map(|h| h + 4).unwrap_or(n);
    let req = String::from_utf8_lossy(&buf[..hend]).to_string();
    let body = String::from_utf8_lossy(&buf[hend..n]).to_string();
    let line = req.lines().next().unwrap_or("");
    let mut it = line.split_whitespace();
    let (method, target) = (it.next().unwrap_or(""), it.next().unwrap_or("/"));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let q = |k: &str| query.split('&').find_map(|p| p.strip_prefix(&format!("{k}="))).map(pct);
    match (method, path) {
        ("GET", "/") => respond(&mut s, "text/html; charset=utf-8", DASH.as_bytes()),
        ("GET", "/state") => { let b = sh.state.lock().unwrap().to_string(); respond(&mut s, "application/json", b.as_bytes()) }
        ("GET", "/pos") => respond(&mut s, "application/octet-stream", &sh.pos),
        ("GET", "/group") => respond(&mut s, "application/octet-stream", &sh.group),
        ("GET", "/region") => respond(&mut s, "application/octet-stream", &sh.region),
        ("GET", "/backbone") => respond(&mut s, "application/octet-stream", &sh.backbone),
        ("GET", "/skel") => {
            // [n_seg u32][pad u32][seg i16 × 6n][pad][neurona u32 × n] — morfología real (Schlegel et al. 2024)
            let lod = if q("lod").as_deref() == Some("small") { "small" } else { "full" };
            let dir = std::env::var("EDI_BRAIN").unwrap_or("/usr/share/edi/brain/flywire783".into());
            match (std::fs::read(format!("{dir}/skel_{lod}.i16")), std::fs::read(format!("{dir}/skel_{lod}.u32"))) {
                (Ok(seg), Ok(nid)) => {
                    let n = (nid.len() / 4) as u32;
                    let pad = (4 - seg.len() % 4) % 4;
                    let mut b = Vec::with_capacity(8 + seg.len() + pad + nid.len());
                    b.extend_from_slice(&n.to_le_bytes()); b.extend_from_slice(&0u32.to_le_bytes());
                    b.extend_from_slice(&seg); b.extend(std::iter::repeat(0u8).take(pad)); b.extend_from_slice(&nid);
                    respond(&mut s, "application/octet-stream", &b)
                }
                _ => { let _ = s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n"); }
            }
        }
        ("GET", "/widget") => respond(&mut s, "text/html; charset=utf-8", WIDGET.as_bytes()),
        ("GET", "/face") => { let b = sh.face.lock().unwrap().to_string(); respond(&mut s, "application/json", b.as_bytes()) }
        ("GET", "/activity") => {
            let a = sh.activity.lock().unwrap();
            let v: Vec<_> = a.iter().map(|(t, r)| serde_json::json!({"t": t, "r": r})).collect();
            respond(&mut s, "application/json", serde_json::json!(v).to_string().as_bytes())
        }
        ("GET", p) if p.starts_with("/face/") && (p.ends_with(".png") || p.ends_with(".svg")) && !p.contains("..") => {
            let ct = if p.ends_with(".svg") { "image/svg+xml" } else { "image/png" };
            let dir = std::env::var("EDI_ASSETS").unwrap_or("/usr/share/edi/face".into());
            match std::fs::read(format!("{dir}/{}", &p[6..])) {
                Ok(b) => { let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nCache-Control: max-age=86400\r\n\r\n", b.len()); let _ = s.write_all(&b); }
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
            // tu frase se registra como turno; EDI la oye en el siguiente tick (50 ms)
            let text: String = body.trim().chars().take(500).collect();
            let txt = q("fmt").as_deref() == Some("text");
            if text.is_empty() { return respond(&mut s, "application/json", b"{\"error\":\"vacio\"}"); }
            if text.starts_with('/') {
                // orden: la contesta el sistema (no el cerebro), al momento
                let id = sh.chat.lock().unwrap().push("tú", &text, "orden", None, vec![]).id;
                let (kind, reply) = crate::uso::command(&sh, &text);
                let r = sh.chat.lock().unwrap().push("sistema", &reply, &kind, Some(id), vec![]);
                if txt { return respond(&mut s, "text/plain; charset=utf-8", format!("{reply}\n").as_bytes()); }
                return respond(&mut s, "application/json", serde_json::json!({"turn": id, "sistema": reply, "kind": kind, "sistema_turn": r.id}).to_string().as_bytes());
            }
            let last = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).last().unwrap_or("").to_lowercase();
            let id = sh.chat.lock().unwrap().push("tú", &text, "frase", None, vec![]).id;
            sh.inbox.lock().unwrap().push((id, text));
            if q("wait").as_deref() == Some("0") {
                return respond(&mut s, "application/json", serde_json::json!({"turn": id}).to_string().as_bytes());
            }
            // por defecto espera su respuesta (huele cada candidata ~0,4 s de tiempo biológico)
            let t0 = std::time::Instant::now();
            let (mut reply, mut fb) = (None, false);
            while t0.elapsed().as_secs_f32() < 10.0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let c = sh.chat.lock().unwrap();
                if c.turns.iter().any(|t| t.id == id && t.kind == "feedback") { fb = true; break; }
                if let Some(t) = c.turns.iter().rev().find(|t| t.reply_to == Some(id) && t.who == "edi") { reply = Some(t.clone()); break; }
            }
            if txt {
                let out = match (&reply, fb) {
                    (Some(r), _) => format!("EDI › {}   [{}]\n", r.text, r.kind),
                    (None, true) => "EDI › (lo toma como tu aprobación de su última respuesta)\n".into(),
                    (None, false) => "EDI › … (sigue pensando; mira /chat)\n".into(),
                };
                return respond(&mut s, "text/plain; charset=utf-8", out.as_bytes());
            }
            let mut v: serde_json::Value = serde_json::from_str(&recall_json(&sh, &last)).unwrap();
            v["turn"] = serde_json::json!(id);
            if let Some(r) = reply { v["edi"] = serde_json::json!(r.text); v["edi_turn"] = serde_json::json!(r.id); v["kind"] = serde_json::json!(r.kind); v["cands"] = serde_json::json!(r.cands); }
            respond(&mut s, "application/json", v.to_string().as_bytes())
        }
        ("POST", "/sense") => {
            // sentidos externos: «CMD:git» (hook de la terminal), «APP:…», «EV:…». ?ok=0 = el comando falló.
            // Solo etiquetas cortas: nunca argumentos ni contenido.
            let ok = q("ok").as_deref() != Some("0");
            let mut n = 0;
            for l in body.lines().map(str::trim).filter(|l| !l.is_empty()).take(16) {
                let Some((k, v)) = l.split_once(':') else { continue };
                let k = k.trim().to_uppercase();
                let v: String = v.trim().chars().filter(|c| !c.is_control()).take(48).collect();
                if v.is_empty() || !["CMD", "APP", "EV"].contains(&k.as_str()) { continue; }
                let (label, val) = if k == "CMD" && !ok { (format!("FALLO:{v}"), -0.3) } else { (format!("{k}:{v}"), 0.05) };
                sh.sensed.lock().unwrap().push((label, val));
                n += 1;
            }
            respond(&mut s, "application/json", serde_json::json!({"ok": n > 0, "n": n}).to_string().as_bytes())
        }
        ("GET", "/diario") => {
            let h: u64 = q("h").and_then(|v| v.parse().ok()).unwrap_or(24);
            let t = crate::uso::diary(&sh, &sh.state_dir, h.clamp(1, 720), crate::now());
            respond(&mut s, "text/plain; charset=utf-8", t.as_bytes())
        }
        ("GET", "/chat") => {
            let since: u64 = q("since").and_then(|v| v.parse().ok()).unwrap_or(0);
            let c = sh.chat.lock().unwrap();
            let turns: Vec<_> = c.turns.iter().filter(|t| t.id > since).cloned().collect();
            // estado reciente de cada turno (👍/👎 y tipo pueden cambiar después de enviarlo)
            let recent: Vec<_> = c.turns.iter().rev().take(40).map(|t| serde_json::json!([t.id, t.reward, t.kind])).collect();
            let th = c.thinking.map(|(t, d, n)| serde_json::json!({"turn": t, "olidas": d, "total": n}));
            respond(&mut s, "application/json", serde_json::json!({"last": c.next, "turns": turns, "recent": recent, "thinking": th}).to_string().as_bytes())
        }
        ("POST", "/reward") => {
            // 👍/👎 a una respuesta concreta (turn=ID) o a la última
            let sign = if body.trim().starts_with('-') || q("v").as_deref() == Some("-") { -1.0 } else { 1.0 };
            let turn = q("turn").and_then(|v| v.parse().ok());
            sh.fb.lock().unwrap().push(crate::talk::Feedback::Reward { turn, sign });
            respond(&mut s, "application/json", b"{\"ok\":true}")
        }
        ("POST", "/teach") => {
            // ✏️ enseñar: «a eso se responde: TEXTO» (turn=ID o la última respuesta)
            // varias líneas «pregunta => respuesta»: un manual de frases de una vez
            let turn = q("turn").and_then(|v| v.parse().ok());
            let pairs: Vec<_> = body.lines().filter_map(crate::talk::split_pair).take(500).collect();
            if pairs.len() > 1 || (pairs.len() == 1 && body.trim().lines().count() > 1) {
                let n = pairs.len();
                let mut f = sh.fb.lock().unwrap();
                for (q, a) in pairs { f.push(crate::talk::Feedback::Pair { q: q.chars().take(300).collect(), a: a.chars().take(300).collect() }); }
                return respond(&mut s, "application/json", serde_json::json!({"ok": true, "pares": n}).to_string().as_bytes());
            }
            let text: String = body.trim().chars().take(300).collect();
            if text.is_empty() { return respond(&mut s, "application/json", b"{\"error\":\"vacio\"}"); }
            sh.fb.lock().unwrap().push(crate::talk::Feedback::Teach { turn, text });
            respond(&mut s, "application/json", b"{\"ok\":true}")
        }
        ("GET", "/chat.js") => respond(&mut s, "text/javascript; charset=utf-8", CHATJS.as_bytes()),
        ("GET", "/learn") => respond(&mut s, "application/json", sh.learn.lock().unwrap().to_string().as_bytes()),
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
