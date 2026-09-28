//! edid — núcleo vivo de EDI.os (v0).
//!
//! Mantiene el cerebro (conectoma LIF) en marcha de forma continua, con:
//! - continuidad: snapshot del estado neuronal + edad; al reiniciar, EDI sigue siendo EDI;
//! - memoria de vida: registro de eventos append-only (events.jsonl);
//! - interocepción: presión de CPU/memoria del kernel (PSI) → energía;
//! - homeostasis: con poca energía se ralentiza; con presión de sueño alta, duerme;
//! - azar físico grabado (cinta de entropía): futuro abierto, pasado reproducible;
//! - estado consultable en 127.0.0.1:7077 (JSON) para el dashboard / Niri / Waybar.

use anyhow::Result;
use edi_brain::{entropy::{Entropy, Source}, Brain, Connectome, Params};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TICK_BIO_MS: f64 = 50.0;
const SNAPSHOT_EVERY: u64 = 40;
const SLEEP_AFTER_TICKS: f64 = 2400.0;

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn psi(kind: &str) -> f64 {
    std::fs::read_to_string(format!("/proc/pressure/{kind}")).ok()
        .and_then(|s| s.lines().next().and_then(|l| l.split_whitespace()
            .find_map(|t| t.strip_prefix("avg10=")).and_then(|x| x.parse().ok())))
        .unwrap_or(0.0)
}

struct Vital {
    age_ticks: u64,
    energy: f64,
    sleep_pressure: f64,
    asleep: bool,
    spikes_last: usize,
    births: u64,
}

fn save(dir: &PathBuf, b: &Brain, v: &Vital) -> Result<()> {
    let mut raw = Vec::with_capacity(b.v.len() * 8 + 32);
    for x in [v.age_ticks, v.births, v.sleep_pressure.to_bits(), v.energy.to_bits()] {
        raw.extend_from_slice(&x.to_le_bytes());
    }
    for x in b.v.iter().chain(&b.g) {
        raw.extend_from_slice(&x.to_le_bytes());
    }
    let tmp = dir.join("snapshot.tmp");
    std::fs::write(&tmp, raw)?;
    std::fs::rename(tmp, dir.join("snapshot.bin"))?;
    Ok(())
}

fn restore(dir: &PathBuf, b: &mut Brain, v: &mut Vital) -> bool {
    let Ok(raw) = std::fs::read(dir.join("snapshot.bin")) else { return false };
    if raw.len() != 32 + b.v.len() * 8 {
        return false;
    }
    let u = |i: usize| u64::from_le_bytes(raw[i * 8..i * 8 + 8].try_into().unwrap());
    (v.age_ticks, v.births, v.sleep_pressure, v.energy) = (u(0), u(1), f64::from_bits(u(2)), f64::from_bits(u(3)));
    let n = b.v.len();
    for i in 0..n {
        b.v[i] = f32::from_le_bytes(raw[32 + i * 4..36 + i * 4].try_into().unwrap());
        b.g[i] = f32::from_le_bytes(raw[32 + (n + i) * 4..36 + (n + i) * 4].try_into().unwrap());
    }
    true
}

fn main() -> Result<()> {
    let brain_dir = PathBuf::from(std::env::var("EDI_BRAIN").unwrap_or("/usr/share/edi/brain/flywire783".into()));
    let state = PathBuf::from(std::env::var("EDI_STATE").unwrap_or_else(|_| {
        format!("{}/.local/state/edi", std::env::var("HOME").unwrap_or("/tmp".into()))
    }));
    let max_ticks: u64 = std::env::var("EDI_MAX_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(u64::MAX);
    std::fs::create_dir_all(&state)?;
    let mut events = std::fs::OpenOptions::new().create(true).append(true).open(state.join("events.jsonl"))?;
    let mut log = |ev: serde_json::Value| { let _ = writeln!(events, "{ev}"); };

    let c = Connectome::load(&brain_dir)?;
    let mut brain = Brain::new(&c, Params { dt: 0.5, p_release: Some(0.5), ..Params::default() });
    let mut vital = Vital { age_ticks: 0, energy: 1.0, sleep_pressure: 0.0, asleep: false, spikes_last: 0, births: 0 };
    let resumed = restore(&state, &mut brain, &mut vital);
    vital.births += 1;
    log(serde_json::json!({"t": now(), "ev": if resumed {"despierta"} else {"nace"}, "age_ticks": vital.age_ticks, "births": vital.births}));
    let src = if std::env::var("EDI_ENTROPY").as_deref() == Ok("fixed") { Source::Fixed(1) } else { Source::Os };
    let mut entropy = Entropy::new(src, Some(state.join("entropy.tape")))?;

    let shared = Arc::new(Mutex::new(serde_json::json!({})));
    let srv = shared.clone();
    std::thread::spawn(move || {
        if let Ok(l) = TcpListener::bind("127.0.0.1:7077") {
            for mut s in l.incoming().flatten() {
                let mut buf = [0u8; 512];
                let _ = s.read(&mut buf);
                let body = srv.lock().unwrap().to_string();
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
            }
        }
    });

    // Actividad espontánea: ruido de fondo leve en el oído (órgano de Johnston).
    let drive: Vec<(u32, f32)> = c.group("auditory_jo").iter().map(|&i| (i, 5.0)).collect();
    let dn = c.group("descending").to_vec();
    let mut ticks_run = 0u64;
    while ticks_run < max_ticks {
        let t0 = Instant::now();
        let cpu = psi("cpu");
        let mem = psi("memory");
        vital.energy = (1.0 - (cpu + mem) / 100.0).clamp(0.0, 1.0);
        if !vital.asleep && vital.sleep_pressure >= 1.0 {
            vital.asleep = true;
            log(serde_json::json!({"t": now(), "ev": "duerme", "age_ticks": vital.age_ticks}));
        }
        if vital.asleep {
            // v0: el sueño descansa y reduce presión (la consolidación SAMN/EGGROLL llegará aquí).
            vital.sleep_pressure = (vital.sleep_pressure - 0.05).max(0.0);
            if vital.sleep_pressure == 0.0 {
                vital.asleep = false;
                log(serde_json::json!({"t": now(), "ev": "despierta_del_sueño", "age_ticks": vital.age_ticks}));
            }
            std::thread::sleep(Duration::from_millis(50));
        } else {
            let mut rng = entropy.rng()?;
            brain.reset_counts();
            brain.run(TICK_BIO_MS, &drive, &mut rng);
            vital.spikes_last = brain.counts.iter().map(|&x| x as usize).sum();
            vital.sleep_pressure += 1.0 / SLEEP_AFTER_TICKS;
        }
        vital.age_ticks += 1;
        ticks_run += 1;
        let dn_active = dn.iter().filter(|&&i| brain.counts[i as usize] > 0).count();
        *shared.lock().unwrap() = serde_json::json!({
            "name": "EDI", "age_bio_s": vital.age_ticks as f64 * TICK_BIO_MS / 1000.0, "births": vital.births,
            "asleep": vital.asleep, "energy": vital.energy, "sleep_pressure": vital.sleep_pressure,
            "spikes_last_tick": vital.spikes_last, "descending_active": dn_active, "psi": {"cpu": cpu, "memory": mem},
        });
        if vital.age_ticks % SNAPSHOT_EVERY == 0 {
            save(&state, &brain, &vital)?;
        }
        // homeostasis: con poca energía cede CPU al usuario
        if vital.energy < 0.7 {
            std::thread::sleep(t0.elapsed().mul_f64(1.0 - vital.energy));
        }
    }
    save(&state, &brain, &vital)?;
    log(serde_json::json!({"t": now(), "ev": "pausa", "age_ticks": vital.age_ticks}));
    Ok(())
}
