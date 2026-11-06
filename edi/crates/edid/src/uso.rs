//! Usos de EDI en el día a día: órdenes del chat («/diario», «/nota», «/recuerdame»…),
//! diario de lo que percibió, notas, recordatorios y vigilancia de puertos.
//!
//! Honesto: estas respuestas las escribe el **sistema** (turnos `who = "sistema"`), no el
//! cerebro. Lo que sí pasa por el cerebro es lo percibido: las notas, los comandos y los
//! puertos entran como eventos (olor, ROSA y memoria SAMN) igual que el resto de sus sentidos.
use crate::http::Shared;
use crate::talk::{plain, Feedback};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::Ordering;

pub const HELP: &str = "Órdenes (empiezan por /):
/diario [horas]        lo que percibí: apps, comandos, puertos, conversación
/nota TEXTO            apunto una nota (y la recuerdo)
/notas [palabra]       tus notas (o las que contienen esa palabra)
/recuerdame 10m TEXTO  aviso dentro de 10 min (también 2h, 1h30m, 18:30)
/recordatorios         los pendientes · /borra N quita el N
/recuerda PALABRA      qué asocio a esa palabra
/enseña HOLA => QUÉ TAL  a eso se responde eso (sin turno previo)
/aprendido             lo que llevo aprendido
/estado                cómo estoy · /duerme me manda a dormir
Atajos: + / - premian o castigan mi última respuesta · = texto me enseña qué debía decir.";

#[derive(Clone, Serialize, Deserialize)]
pub struct Reminder { pub id: u64, pub at: u64, pub text: String, #[serde(default)] pub done: bool }

pub fn load_reminders(dir: &Path) -> Vec<Reminder> {
    std::fs::read(dir.join("reminders.json")).ok().and_then(|r| serde_json::from_slice(&r).ok()).unwrap_or_default()
}
pub fn save_reminders(dir: &Path, r: &[Reminder]) {
    let keep: Vec<&Reminder> = r.iter().filter(|x| !x.done).collect();
    let _ = std::fs::write(dir.join("reminders.tmp"), serde_json::to_vec(&keep).unwrap_or_default())
        .and_then(|_| std::fs::rename(dir.join("reminders.tmp"), dir.join("reminders.json")));
}

/// «10m», «2h», «1h30m», «90s», «45» (min), «en 10 minutos», «18:30» (hoy o mañana), «@unix».
/// Devuelve (instante unix, palabras consumidas).
pub fn parse_when(ws: &[&str], now: u64, offset: i64) -> Option<(u64, usize)> {
    let mut i = 0;
    if ws.first().map(|w| plain(w)) == Some("en".into()) || ws.first().map(|w| plain(w)) == Some("a".into()) { i = 1; }
    if ws.get(i).map(|w| plain(w)) == Some("las".into()) || ws.get(i).map(|w| plain(w)) == Some("la".into()) { i += 1; }
    let w = *ws.get(i)?;
    if let Some(u) = w.strip_prefix('@') { return u.parse().ok().map(|t| (t, i + 1)); }
    if let Some((h, m)) = w.split_once(':') {
        let (h, m): (i64, i64) = (h.parse().ok()?, m.parse().ok()?);
        if !(0..24).contains(&h) || !(0..60).contains(&m) { return None; }
        let local = now as i64 + offset;
        let mut at = local - local.rem_euclid(86400) + h * 3600 + m * 60;
        if at <= local { at += 86400; }
        return Some(((at - offset) as u64, i + 1));
    }
    // número + unidad pegada o en la palabra siguiente
    let unit = |u: &str| -> Option<u64> {
        let u = plain(u);
        match u.as_str() {
            "s" | "seg" | "segs" | "segundo" | "segundos" => Some(1),
            "" | "m" | "min" | "mins" | "minuto" | "minutos" => Some(60),
            "h" | "hora" | "horas" => Some(3600),
            "d" | "dia" | "dias" => Some(86400),
            _ => None,
        }
    };
    let (mut secs, mut num, mut any) = (0u64, String::new(), false);
    let mut chars = w.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() { num.push(c); continue; }
        let mut u = c.to_string();
        while let Some(&d) = chars.peek() { if d.is_ascii_digit() { break; } u.push(d); chars.next(); }
        secs += num.parse::<u64>().ok()? * unit(&u)?;
        num.clear();
        any = true;
    }
    let mut used = i + 1;
    if !num.is_empty() {
        // «10 minutos»: la unidad es la palabra siguiente
        let n: u64 = num.parse().ok()?;
        match ws.get(used).and_then(|u| unit(u)) { Some(k) if !ws[used].is_empty() => { secs += n * k; used += 1; } _ => secs += n * 60 }
        any = true;
    }
    (any && secs > 0).then(|| (now + secs, used))
}

pub fn dur(s: u64) -> String {
    if s < 60 { format!("{s} s") } else if s < 3600 { format!("{} min", (s + 30) / 60) }
    else if s < 86400 { let m = (s + 30) / 60; if m % 60 == 0 { format!("{} h", m / 60) } else { format!("{} h {} min", m / 60, m % 60) } }
    else { format!("{} d {} h", s / 86400, (s % 86400) / 3600) }
}

pub fn hhmm(t: u64, offset: i64) -> String { let l = (t as i64 + offset).rem_euclid(86400); format!("{:02}:{:02}", l / 3600, (l % 3600) / 60) }

/// Últimas líneas del registro de vida (events.jsonl), como mucho los últimos 8 MB.
fn read_events(dir: &Path) -> Vec<(u64, String)> {
    let Ok(mut f) = std::fs::File::open(dir.join("events.jsonl")) else { return vec![] };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(8 << 20);
    let _ = f.seek(SeekFrom::Start(start));
    let mut s = String::new();
    let _ = f.read_to_string(&mut s);
    s.lines().skip(if start > 0 { 1 } else { 0 }).filter_map(|l| {
        let v: serde_json::Value = serde_json::from_str(l).ok()?;
        Some((v["t"].as_u64()?, v["ev"].as_str()?.to_string()))
    }).collect()
}

fn top(m: &HashMap<String, u64>, n: usize, f: impl Fn(&str, u64) -> String) -> String {
    let mut v: Vec<(&String, &u64)> = m.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    v.iter().take(n).map(|(k, x)| f(k, **x)).collect::<Vec<_>>().join(" · ")
}

/// El diario: lo que EDI percibió en las últimas `hours` horas (datos reales de sus sentidos).
pub fn diary(sh: &Shared, dir: &Path, hours: u64, now: u64) -> String {
    let since = now.saturating_sub(hours * 3600);
    let ev: Vec<(u64, String)> = read_events(dir).into_iter().filter(|(t, _)| *t >= since).collect();
    let (mut apps, mut cmds, mut fails, mut words) = (HashMap::new(), HashMap::new(), 0u64, HashMap::new());
    let (mut ports, mut sleeps, mut alive) = (vec![], 0, 0u64);
    // tiempo en primer plano: de un FOCO al siguiente (máx. 30 min: si no, estabas fuera)
    let mut cur: Option<(String, u64)> = None;
    let close = |cur: &mut Option<(String, u64)>, t: u64, apps: &mut HashMap<String, u64>| {
        if let Some((a, t0)) = cur.take() { *apps.entry(a).or_insert(0) += t.saturating_sub(t0).min(1800); }
    };
    let mut last_t = since;
    for (t, e) in &ev {
        if t.saturating_sub(last_t) < 600 { alive += t - last_t; }
        last_t = *t;
        if let Some(a) = e.strip_prefix("FOCO:") { close(&mut cur, *t, &mut apps); cur = Some((a.to_string(), *t)); }
        else if e == "pausa" { close(&mut cur, *t, &mut apps); }
        else if let Some(c) = e.strip_prefix("CMD:") { *cmds.entry(c.to_string()).or_insert(0u64) += 1; }
        else if let Some(c) = e.strip_prefix("FALLO:") { *cmds.entry(c.to_string()).or_insert(0u64) += 1; fails += 1; }
        else if let Some(p) = e.strip_prefix("PORT:") { if !ports.contains(&p.to_string()) { ports.push(p.to_string()); } }
        else if let Some(w) = e.strip_prefix("OYE:") { *words.entry(w.to_string()).or_insert(0u64) += 1; }
        else if e.starts_with("duerme") { sleeps += 1; }
    }
    close(&mut cur, now.min(last_t + 1800), &mut apps);
    let c = sh.chat.lock().unwrap();
    let ts: Vec<_> = c.turns.iter().filter(|t| t.t >= since).collect();
    let (tu, edi) = (ts.iter().filter(|t| t.who == "tú" && t.kind == "frase").count(), ts.iter().filter(|t| t.who == "edi").count());
    let (up, down) = (ts.iter().filter(|t| t.who == "edi" && t.reward > 0).count(), ts.iter().filter(|t| t.who == "edi" && t.reward < 0).count());
    let ens = ts.iter().filter(|t| t.kind == "enseñanza" && t.who == "tú").count();
    drop(c);
    let notes = read_notes(dir).iter().filter(|(t, _)| *t >= since).count();
    let mut out = vec![format!("📔 Diario · últimas {hours} h (encendida ~{})", dur(alive))];
    if apps.is_empty() { out.push("Apps: no he visto ventanas (en Linux necesito Niri; en Windows, la ventana activa).".into()); }
    else { out.push(format!("Apps en primer plano (aprox.): {}", top(&apps, 6, |k, s| format!("{k} {}", dur(s))))); }
    if !cmds.is_empty() { out.push(format!("Comandos: {}{}", top(&cmds, 8, |k, n| format!("{k} ×{n}")), if fails > 0 { format!(" ({fails} fallaron)") } else { String::new() })); }
    else { out.push("Comandos: ninguno (instala el hook: «edi hook bash», «edi hook zsh» o «edi.cmd hook»).".into()); }
    out.push(format!("Puertos nuevos: {}", if ports.is_empty() { "ninguno".into() } else { ports.join(" · ") }));
    let pl = |n: usize, s: &str, p: &str| format!("{n} {}", if n == 1 { s } else { p });
    out.push(format!("Conversación: {} tuyas · {} mías · 👍 {up} · 👎 {down} · {} · {}", pl(tu, "frase", "frases"), pl(edi, "respuesta", "respuestas"), pl(ens, "enseñanza", "enseñanzas"), pl(notes, "nota", "notas")));
    if !words.is_empty() { out.push(format!("Lo que más te oí: {}", top(&words, 8, |k, n| if n > 1 { format!("{k} ×{n}") } else { k.to_string() }))); }
    if sleeps > 0 { out.push(format!("Siestas: {sleeps} (cada una consolida la memoria, sueña y olvida).")); }
    out.join("\n")
}

fn notes_path(dir: &Path) -> std::path::PathBuf { dir.join("notes.jsonl") }

fn read_notes(dir: &Path) -> Vec<(u64, String)> {
    std::fs::read_to_string(notes_path(dir)).unwrap_or_default().lines().filter_map(|l| {
        let v: serde_json::Value = serde_json::from_str(l).ok()?;
        Some((v["t"].as_u64()?, v["text"].as_str()?.to_string()))
    }).collect()
}

fn recall_text(sh: &Shared, word: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(&crate::http::recall_json(sh, word)).unwrap_or_default();
    if !v["known"].as_bool().unwrap_or(false) { return format!("«{word}» no me suena de nada."); }
    let assoc: Vec<String> = v["recalled"].as_array().map(|a| a.iter().filter_map(|x| x["label"].as_str())
        .map(|l| l.split_once(':').map(|x| x.1).unwrap_or(l).to_string()).filter(|l| l != word && !l.starts_with("ep")).take(8).collect()).unwrap_or_default();
    let mut assoc2: Vec<String> = vec![];
    for a in assoc { if !assoc2.contains(&a) { assoc2.push(a); } }
    let after: Vec<String> = v["despues"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).map(|l| l.split_once(':').map(|x| x.1).unwrap_or(l).to_string()).collect()).unwrap_or_default();
    let mut s = format!("«{word}» me suena a: {}", if assoc2.is_empty() { "nada más".into() } else { assoc2.join(", ") });
    if !after.is_empty() { s += &format!("\nLa última vez, después vino: {}", after.join(" → ")); }
    if let Some(val) = v["valencia"].as_f64() { s += &format!("\nValencia (sinapsis KC→MBON): {val:+.3} {}", if val > 0.02 { "· me gusta" } else if val < -0.02 { "· no me gusta" } else { "· neutra" }); }
    s
}

/// Ejecuta una orden del chat. Devuelve (tipo, texto del sistema).
pub fn command(sh: &Shared, line: &str) -> (String, String) {
    let now = crate::now();
    let dir = sh.state_dir.as_path();
    let line = line.trim().trim_start_matches('/');
    let (cmd, rest) = line.split_once(char::is_whitespace).map(|(a, b)| (a, b.trim())).unwrap_or((line, ""));
    let off = sh.tz_offset;
    let k = |s: &str| s.to_string();
    match plain(cmd).as_str() {
        "ayuda" | "help" | "?" | "" => (k("ayuda"), HELP.into()),
        "diario" | "hoy" => {
            let h = rest.trim_end_matches('h').parse().unwrap_or(24u64).clamp(1, 24 * 30);
            (k("diario"), diary(sh, dir, h, now))
        }
        "nota" | "apunta" => {
            if rest.is_empty() { return (k("ayuda"), "Uso: /nota TEXTO".into()); }
            let line = serde_json::json!({"t": now, "text": rest}).to_string();
            let ok = std::fs::OpenOptions::new().create(true).append(true).open(notes_path(dir))
                .and_then(|mut f| { use std::io::Write; writeln!(f, "{line}") }).is_ok();
            // la nota también la «oye»: entra como evento (memoria) y sus palabras al oído
            sh.sensed.lock().unwrap().push((format!("NOTA:{}", rest.chars().take(60).collect::<String>()), 0.2));
            sh.inbox_quiet.lock().unwrap().push(rest.to_string());
            let n = read_notes(dir).len();
            (k("nota"), if ok { format!("📝 Apuntado. Llevas {n} nota{} (/notas para verlas).", if n == 1 { "" } else { "s" }) } else { "No he podido guardar la nota.".into() })
        }
        "notas" => {
            let q = plain(rest);
            let ns: Vec<(u64, String)> = read_notes(dir).into_iter().filter(|(_, t)| q.is_empty() || plain(t).contains(&q)).collect();
            if ns.is_empty() { return (k("nota"), if q.is_empty() { "No tienes notas. /nota TEXTO para apuntar una.".into() } else { format!("Ninguna nota con «{rest}».") }); }
            let s = ns.iter().rev().take(12).map(|(t, x)| format!("· {} (hace {}) {x}", hhmm(*t, off), dur(now.saturating_sub(*t)))).collect::<Vec<_>>().join("\n");
            (k("nota"), format!("📝 {} nota{}:\n{s}", ns.len(), if ns.len() == 1 { "" } else { "s" }))
        }
        "recuerdame" | "avisame" | "recordatorio" => {
            let ws: Vec<&str> = rest.split_whitespace().collect();
            let Some((at, used)) = parse_when(&ws, now, off) else { return (k("ayuda"), "Uso: /recuerdame 10m TEXTO (también 2h, 1h30m, «en 20 minutos», 18:30)".into()) };
            let text = ws[used..].join(" ");
            let text = if text.is_empty() { "¡Lo que me pediste!".to_string() } else { text };
            let mut r = sh.reminders.lock().unwrap();
            let id = r.iter().map(|x| x.id).max().unwrap_or(0) + 1;
            r.push(Reminder { id, at, text: text.clone(), done: false });
            save_reminders(dir, &r);
            (k("recordatorio"), format!("⏰ Te aviso a las {} (en {}): {text}", hhmm(at, off), dur(at.saturating_sub(now))))
        }
        "recordatorios" | "pendientes" => {
            let r = sh.reminders.lock().unwrap();
            let p: Vec<_> = r.iter().filter(|x| !x.done).collect();
            if p.is_empty() { return (k("recordatorio"), "No tienes recordatorios pendientes.".into()); }
            (k("recordatorio"), p.iter().map(|x| format!("{} · {} (en {}) {}", x.id, hhmm(x.at, off), dur(x.at.saturating_sub(now)), x.text)).collect::<Vec<_>>().join("\n"))
        }
        "borra" | "cancela" => {
            let mut r = sh.reminders.lock().unwrap();
            let id: u64 = rest.trim_start_matches('#').parse().unwrap_or(0);
            match r.iter_mut().find(|x| x.id == id && !x.done) {
                Some(x) => { x.done = true; let t = x.text.clone(); save_reminders(dir, &r); (k("recordatorio"), format!("Quitado: {t}")) }
                None => (k("recordatorio"), format!("No hay recordatorio {id} pendiente (/recordatorios).")),
            }
        }
        "recuerda" | "asocia" => {
            let w = rest.split(|c: char| !c.is_alphanumeric()).find(|w| !w.is_empty()).unwrap_or("").to_lowercase();
            if w.is_empty() { return (k("ayuda"), "Uso: /recuerda PALABRA".into()); }
            (k("recuerdo"), recall_text(sh, &w))
        }
        "ensena" | "enseña" | "aprende" => match crate::talk::split_pair(rest) {
            Some((q, a)) => { sh.fb.lock().unwrap().push(Feedback::Pair { q: q.clone(), a: a.clone() }); (k("enseñanza"), format!("✏️ Aprendido: a «{q}» respondo «{a}».")) }
            None => (k("ayuda"), "Uso: /enseña PREGUNTA => RESPUESTA".into()),
        },
        "duerme" | "dormir" => { sh.force_sleep.store(true, Ordering::Relaxed); (k("estado"), "Me voy a dormir: consolido, sueño y olvido. Vuelvo enseguida.".into()) }
        "estado" => {
            let s = sh.state.lock().unwrap().clone();
            (k("estado"), format!("{} · edad {} · nacimiento nº {} · energía {:.0} % · sueño {:.0} % · {} recuerdos · ×{:.1} tiempo real",
                if s["asleep"].as_bool().unwrap_or(false) { "Durmiendo" } else { "Despierta" }, dur(s["age_bio_s"].as_f64().unwrap_or(0.0) as u64),
                s["births"], 100.0 * s["energy"].as_f64().unwrap_or(0.0), 100.0 * s["sleep_pressure"].as_f64().unwrap_or(0.0).min(1.0),
                s["memory"]["nodes"], s["speed_x_realtime"].as_f64().unwrap_or(0.0)))
        }
        "aprendido" | "aprende?" | "progreso" => {
            let l = sh.learn.lock().unwrap().clone();
            let h = &l["habla"];
            (k("estado"), format!("Habla: {} palabras · {} frases dichas · {} enseñanzas · {} refuerzos · {} turnos en mi memoria de conversación\nPredicción (ROSA): acierto {:.0} % sobre {} conceptos · espero «{}»\nDopamina: {} refuerzos → {} sinapsis KC→MBON deprimidas",
                h["palabras"], h["frases_dichas"], h["ensenanzas"], h["refuerzos"], h["turnos_oidos_y_dichos"],
                100.0 * l["rosa"]["acierto_prediccion"].as_f64().unwrap_or(0.0), l["rosa"]["conceptos_vistos"], l["rosa"]["espera"].as_str().unwrap_or("nada"),
                l["dopamina"]["refuerzos"], l["dopamina"]["sinapsis_kc_mbon_deprimidas"]))
        }
        _ => (k("ayuda"), format!("No conozco «/{cmd}».\n{HELP}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cuando() {
        let now = 1_000_000;
        fn w(s: &str) -> Vec<&str> { s.split_whitespace().collect() }
        assert_eq!(parse_when(&w("10m llamar"), now, 0), Some((now + 600, 1)));
        assert_eq!(parse_when(&w("1h30m x"), now, 0), Some((now + 5400, 1)));
        assert_eq!(parse_when(&w("en 20 minutos sacar la pizza"), now, 0), Some((now + 1200, 3)));
        assert_eq!(parse_when(&w("45 algo"), now, 0), Some((now + 2700, 1)));
        assert_eq!(parse_when(&w("2 horas"), now, 0), Some((now + 7200, 2)));
        assert_eq!(parse_when(&w("90s"), now, 0), Some((now + 90, 1)));
        assert_eq!(parse_when(&w("mañana"), now, 0), None);
        // 18:30 en UTC+1 con now = 00:00 UTC (01:00 local) → 17:30 UTC del mismo día
        let day = 86400 * 20000;
        assert_eq!(parse_when(&w("a las 18:30 cena"), day, 3600), Some((day + 17 * 3600 + 1800, 3)));
        // si ya pasó hoy, mañana
        assert_eq!(parse_when(&w("00:30"), day + 3 * 3600, 0), Some((day + 86400 + 1800, 1)));
    }
    #[test]
    fn duraciones() {
        assert_eq!(dur(42), "42 s");
        assert_eq!(dur(600), "10 min");
        assert_eq!(dur(5400), "1 h 30 min");
        assert_eq!(dur(7200), "2 h");
    }
}
