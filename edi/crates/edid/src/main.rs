//! edid — núcleo vivo de EDI.os.
//!
//! Un solo daemon dueño de: el cerebro (conectoma LIF continuo, sinapsis estocásticas),
//! la memoria episódica real (SAMN), la homeostasis (PSI del kernel = interocepción),
//! el sueño (consolidación NREM, sueños REM, olvido), la continuidad (snapshot + log),
//! los sentidos del sistema (procesos, puertos, ventana enfocada, palabras que le dices)
//! y su cuerpo en el escritorio (Niri cambia de color con su estado). Sirve el cerebro
//! en 3D en http://127.0.0.1:7077

mod face;
mod http;
mod mood;
mod senses;

use anyhow::Result;
use edi_brain::{entropy::{Entropy, Source}, Brain, Connectome, Params};
use edi_samn::{Samn, EMB};
use http::Shared;
use std::collections::{HashSet, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TICK_BIO_MS: f64 = 50.0;
const SNAPSHOT_EVERY: u64 = 40;
const SENSE_EVERY: u64 = 12;
const EPISODE_IDLE: u64 = 60;
const EPISODE_MAX: u64 = 500;

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }
fn env<T: std::str::FromStr>(k: &str, d: T) -> T { std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d) }

struct Vital { age_ticks: u64, energy: f64, sleep_pressure: f64, asleep: bool, spikes_last: usize, births: u64 }

fn save(dir: &PathBuf, b: &Brain, v: &Vital, m: &Samn) -> Result<()> {
    let mut raw = Vec::with_capacity(b.v.len() * 8 + 32);
    for x in [v.age_ticks, v.births, v.sleep_pressure.to_bits(), v.energy.to_bits()] { raw.extend_from_slice(&x.to_le_bytes()); }
    for x in b.v.iter().chain(&b.g) { raw.extend_from_slice(&x.to_le_bytes()); }
    let tmp = dir.join("snapshot.tmp");
    std::fs::write(&tmp, raw)?;
    std::fs::rename(tmp, dir.join("snapshot.bin"))?;
    m.save(&dir.join("samn.json"))?;
    Ok(())
}

fn restore(dir: &PathBuf, b: &mut Brain, v: &mut Vital) -> bool {
    let Ok(raw) = std::fs::read(dir.join("snapshot.bin")) else { return false };
    if raw.len() != 32 + b.v.len() * 8 { return false; }
    let u = |i: usize| u64::from_le_bytes(raw[i * 8..i * 8 + 8].try_into().unwrap());
    (v.age_ticks, v.births, v.sleep_pressure, v.energy) = (u(0), u(1), f64::from_bits(u(2)), f64::from_bits(u(3)));
    let n = b.v.len();
    for i in 0..n {
        b.v[i] = f32::from_le_bytes(raw[32 + i * 4..36 + i * 4].try_into().unwrap());
        b.g[i] = f32::from_le_bytes(raw[32 + (n + i) * 4..36 + (n + i) * 4].try_into().unwrap());
    }
    true
}

/// Huella neuronal: actividad de las descendentes en 16 cubetas (normalizada).
fn embed(counts: &[u32], dn: &[u32]) -> [f32; EMB] {
    let mut e = [0f32; EMB];
    for (k, &i) in dn.iter().enumerate() { e[k % EMB] += counts[i as usize] as f32; }
    let s: f32 = e.iter().sum();
    if s > 0.0 { e.iter_mut().for_each(|x| *x /= s); }
    e
}

fn fnv(s: &str) -> u64 { s.bytes().fold(0xcbf29ce484222325, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3)) }

fn positions(dir: &PathBuf) -> (Vec<u8>, f32) {
    let raw = std::fs::read(dir.join("pos.f32")).unwrap_or_default();
    let v: Vec<f32> = raw.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in v.chunks_exact(3).filter(|p| p[0].is_finite()) { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
    let ctr: Vec<f32> = (0..3).map(|k| (lo[k] + hi[k]) / 2.0).collect();
    let sc = (0..3).map(|k| hi[k] - lo[k]).fold(1.0f32, f32::max) / 2.0;
    let bytes = v.chunks_exact(3).flat_map(|p| {
        let q: [f32; 3] = if p[0].is_finite() { [(p[0] - ctr[0]) / sc, -(p[1] - ctr[1]) / sc, (p[2] - ctr[2]) / sc] } else { [f32::NAN; 3] };
        q.into_iter().flat_map(|x| x.to_le_bytes())
    }).collect();
    (bytes, sc / 1000.0)
}

fn main() -> Result<()> {
    let brain_dir = PathBuf::from(std::env::var("EDI_BRAIN").unwrap_or("/usr/share/edi/brain/flywire783".into()));
    let home = std::env::var("HOME").unwrap_or("/tmp".into());
    let state = PathBuf::from(std::env::var("EDI_STATE").unwrap_or_else(|_| format!("{home}/.local/state/edi")));
    let niri_dir = PathBuf::from(std::env::var("EDI_NIRI_DIR").unwrap_or_else(|_| format!("{home}/.config/niri")));
    let (max_ticks, port, sleep_after): (u64, u16, f64) = (env("EDI_MAX_TICKS", u64::MAX), env("EDI_PORT", 7077), env("EDI_SLEEP_AFTER", 2400.0));
    std::fs::create_dir_all(&state)?;
    let mut evlog = std::fs::OpenOptions::new().create(true).append(true).open(state.join("events.jsonl"))?;

    let c = Connectome::load(&brain_dir)?;
    let mut brain = Brain::new(&c, Params { dt: 0.5, p_release: Some(0.5), ..Params::default() });
    let mut vital = Vital { age_ticks: 0, energy: 1.0, sleep_pressure: 0.0, asleep: false, spikes_last: 0, births: 0 };
    let resumed = restore(&state, &mut brain, &mut vital);
    vital.births += 1;
    let samn = if resumed { Samn::load(&state.join("samn.json")).unwrap_or_else(|_| Samn::new()) } else { Samn::new() };

    let mut group = vec![0u8; c.n];
    for (gid, name) in [(1u8, "auditory_jo"), (5, "photoreceptor"), (6, "kenyon_cell"), (7, "mbon"), (8, "dan"), (3, "motor"), (4, "sugar_grn"), (2, "descending")] {
        for &i in c.group(name) { group[i as usize] = gid; }
    }
    let (pos, um_per_unit) = positions(&brain_dir);
    let region: Vec<u8> = std::fs::read(brain_dir.join("region.u8")).ok().filter(|r| r.len() == c.n).unwrap_or(vec![0; c.n]);
    const RN: [&str; 8] = ["otras", "lóbulo óptico", "cuerpo fungiforme", "complejo central", "lóbulo antenal", "cuerno lateral", "SEZ", "vía auditiva"];
    let backbone: Vec<u8> = {
        let valid = |i: usize| f32::from_le_bytes(pos[i * 12..i * 12 + 4].try_into().unwrap()).is_finite();
        let mut e: Vec<(i32, u32, u32)> = Vec::new();
        for pre in 0..c.n {
            if !valid(pre) { continue; }
            for k in c.row_ptr[pre] as usize..c.row_ptr[pre + 1] as usize {
                let w = c.w[k].abs();
                if w >= 12 && valid(c.col[k] as usize) { e.push((w, pre as u32, c.col[k])); }
            }
        }
        e.sort_by(|a, b| b.0.cmp(&a.0));
        e.truncate(60_000);
        e.iter().flat_map(|&(_, a, b)| a.to_le_bytes().into_iter().chain(b.to_le_bytes())).collect()
    };
    for (i, ch) in pos.chunks_exact(12).enumerate() { if f32::from_le_bytes(ch[..4].try_into().unwrap()).is_nan() { group[i] = 255; } }

    let sh = Arc::new(Shared {
        state: Mutex::new(serde_json::json!({})), spikes: Mutex::new(VecDeque::new()), samn: Mutex::new(samn),
        events: Mutex::new(VecDeque::new()), inbox: Mutex::new(vec![]), force_sleep: Default::default(), pos, group,
        region: region.clone(), backbone, activity: Mutex::new(VecDeque::new()), face: Mutex::new(serde_json::json!({})),
    });
    let mut say = |sh: &Shared, t: u64, s: String| {
        let line = serde_json::json!({"t": t, "ev": s});
        let _ = writeln!(evlog, "{line}");
        let mut e = sh.events.lock().unwrap();
        e.push_back(s);
        if e.len() > 60 { e.pop_front(); }
    };
    say(&sh, now(), format!("{} (edad {} ticks, nacimiento nº {})", if resumed { "despierta" } else { "nace" }, vital.age_ticks, vital.births));
    http::serve(sh.clone(), port);

    let src = if std::env::var("EDI_ENTROPY").as_deref() == Ok("fixed") { Source::Fixed(1) } else { Source::Os };
    let mut entropy = Entropy::new(src, Some(state.join("entropy.tape")))?;
    let jo = c.group("auditory_jo").to_vec();
    let dn = c.group("descending").to_vec();
    let baseline: Vec<(u32, f32)> = jo.iter().map(|&i| (i, 5.0)).collect();
    let mut boosts: Vec<(Vec<(u32, f32)>, u32)> = vec![];
    let (mut known_procs, mut known_ports, mut last_focus): (HashSet<String>, HashSet<u16>, String) = (senses::procs(), senses::ports(), String::new());
    let niri_ok = niri_dir.is_dir();
    let (mut ep_start, mut last_event_tick, mut emb_avg) = (0u64, 0u64, [0f32; EMB]);
    let mut ticks_run = 0u64;
    let mut speed = 0.0f64;
    let mut mood = face::Mood::new(vital.age_ticks);
    let mut spike_avg = 0f64;
    let (mut reg_hist, n_reg): (VecDeque<[u32; 8]>, [u32; 8]) = (VecDeque::new(), {
        let mut k = [0u32; 8]; for &r in &region { k[r as usize % 8] += 1; } k });
    let mut metrics = serde_json::json!({});

    while ticks_run < max_ticks {
        let t0 = Instant::now();
        let (cpu, mem) = (senses::psi("cpu"), senses::psi("memory"));
        vital.energy = (1.0 - (cpu + mem) / 100.0).clamp(0.0, 1.0);
        let tk = vital.age_ticks;
        let mut new_events: Vec<(String, f32)> = vec![];

        // sentidos del sistema
        if tk % SENSE_EVERY == 0 {
            let p = senses::procs();
            for n in p.difference(&known_procs).take(4) { new_events.push((format!("PROC:{n}"), 0.0)); }
            if tk > 200 { if let Some(n) = p.difference(&known_procs).next() { mood.stim(tk, face::Stim::Proc(n.clone())); } }
            known_procs = p;
            let q = senses::ports();
            for n in q.difference(&known_ports).take(3) { new_events.push((format!("PORT:{n}"), 0.1)); mood.stim(tk, face::Stim::Port(*n)); }
            known_ports = q;
            if let Some(f) = senses::focused() { if f != last_focus { new_events.push((format!("FOCO:{f}"), 0.0)); mood.stim(tk, face::Stim::Focus(f.clone())); last_focus = f; } }
            if vital.energy < 0.5 { new_events.push(("ESTRES:sistema".into(), -0.5)); }
            if niri_ok { let _ = mood::write(&niri_dir, vital.asleep, vital.energy); }
        }
        // palabras que le dices: memoria + estímulo al oído (órgano de Johnston)
        let heard: Vec<String> = std::mem::take(&mut *sh.inbox.lock().unwrap());
        let mut words = vec![];
        for text in &heard {
            for w in text.split(|c: char| !c.is_alphanumeric()).filter(|w| w.len() > 1) {
                let w = w.to_lowercase();
                let h = fnv(&w);
                let set: Vec<(u32, f32)> = (0..24).map(|k| (jo[((h >> 3).wrapping_add(k * 2654435761) as usize) % jo.len()], 150.0)).collect();
                boosts.push((set, 8));
                new_events.push((format!("OYE:{w}"), 0.3));
                let (known, assoc) = {
                    let mut m = sh.samn.lock().unwrap();
                    match m.find(edi_samn::Kind::Word, &w) {
                        Some(id) => { let r = m.recall(&[(id, 1.0)], None); (true, r.iter().skip(1).take(4).map(|(i, _)| m.nodes[*i as usize].label.replace("OYE:", "")).collect()) }
                        None => (false, vec![]),
                    }
                };
                mood.stim(tk, face::Stim::Word { w: w.clone(), known, assoc });
                words.push(w);
            }
        }

        // sueño: consolidar, soñar, olvidar
        if !vital.asleep && (vital.sleep_pressure >= 1.0 || sh.force_sleep.swap(false, Ordering::Relaxed)) {
            vital.asleep = true;
            let r = sh.samn.lock().unwrap().sleep();
            say(&sh, now(), format!("duerme: {} conceptos, {} habilidades, {} sueños, {} olvidos", r.concepts, r.skills, r.dreamed, r.pruned));
        }
        let tick_spikes: Vec<u32>;
        if vital.asleep {
            vital.sleep_pressure = (vital.sleep_pressure - 0.05).max(0.0);
            if vital.sleep_pressure == 0.0 { vital.asleep = false; say(&sh, now(), "despierta del sueño".into()); }
            std::thread::sleep(Duration::from_millis(50));
            tick_spikes = vec![];
        } else {
            let mut drive = baseline.clone();
            boosts.retain_mut(|(set, left)| { drive.extend(set.iter().cloned()); *left -= 1; *left > 0 });
            let mut rng = entropy.rng()?;
            brain.reset_counts();
            brain.run(TICK_BIO_MS, &drive, &mut rng);
            vital.spikes_last = brain.counts.iter().map(|&x| x as usize).sum();
            vital.sleep_pressure += 1.0 / sleep_after;
            tick_spikes = brain.counts.iter().enumerate().filter(|(_, &c)| c > 0).map(|(i, _)| i as u32).collect();
            let mut rc = [0u32; 8];
            for &i in &tick_spikes { rc[region[i as usize] as usize % 8] += brain.counts[i as usize]; }
            reg_hist.push_back(rc);
            if reg_hist.len() > 200 { reg_hist.pop_front(); }
            let sp = vital.spikes_last as f64;
            if spike_avg > 0.0 && sp > 2.5 * spike_avg + 30.0 {
                let top = (1..8).max_by_key(|&k| rc[k] * 1000 / n_reg[k].max(1)).unwrap_or(0);
                mood.stim(tk, face::Stim::BrainBurst(RN[top]));
            }
            spike_avg = 0.95 * spike_avg + 0.05 * sp;
            // métricas: tasa, sinapsis activas, entropía regional, complejidad LZ, sincronía
            let syn: u64 = tick_spikes.iter().map(|&i| (c.row_ptr[i as usize + 1] - c.row_ptr[i as usize]) as u64).sum();
            let tot: f64 = rc.iter().map(|&x| x as f64).sum::<f64>().max(1.0);
            let ent: f64 = rc.iter().filter(|&&x| x > 0).map(|&x| { let p = x as f64 / tot; -p * p.log2() }).sum();
            let win: Vec<&[u32; 8]> = reg_hist.iter().rev().take(40).collect();
            let mut bits = String::new();
            for k in 0..8 {
                let mean = win.iter().map(|r| r[k] as f64).sum::<f64>() / win.len().max(1) as f64;
                for r in &win { bits.push(if (r[k] as f64) > mean { '1' } else { '0' }); }
            }
            let lz = lz76(bits.as_bytes()) as f64 * (bits.len() as f64).log2() / bits.len().max(1) as f64;
            let rates: Vec<f64> = (1..8).map(|k| rc[k] as f64 / n_reg[k].max(1) as f64).collect();
            let mu = rates.iter().sum::<f64>() / 7.0;
            let sd = (rates.iter().map(|r| (r - mu).powi(2)).sum::<f64>() / 7.0).sqrt();
            metrics = serde_json::json!({"spikes_s": sp / (TICK_BIO_MS / 1000.0), "active_syn_pct": 100.0 * syn as f64 / c.meta.n_edges as f64,
                "entropy_bits": ent, "lz": lz, "sync": if mu > 0.0 { 1.0 / (1.0 + sd / mu) } else { 0.0 }});
            let e = embed(&brain.counts, &dn);
            for k in 0..EMB { emb_avg[k] = 0.9 * emb_avg[k] + 0.1 * e[k]; }
            speed = TICK_BIO_MS / (t0.elapsed().as_secs_f64() * 1000.0);
        }

        // memoria episódica real
        {
            let mut m = sh.samn.lock().unwrap();
            m.tick();
            if !vital.asleep {
                for (lbl, v) in &new_events { m.event(lbl, &emb_avg, *v); last_event_tick = tk; }
                for w in &words { let id = m.observe(edi_samn::Kind::Word, w, Some(&emb_avg), 0.3, edi_samn::Src::Measured); if let Some(ev) = m.find(edi_samn::Kind::Event, &format!("OYE:{w}")) { m.link(id, ev, edi_samn::Rel::Grounds, 1.0); m.link(ev, id, edi_samn::Rel::Grounds, 1.0); } }
                if m.in_episode() && ((tk - last_event_tick > EPISODE_IDLE && new_events.is_empty()) || tk - ep_start > EPISODE_MAX) { m.end_episode(); ep_start = tk; }
                if !m.in_episode() && !new_events.is_empty() { ep_start = tk; }
                if tk % SENSE_EVERY == 0 && !new_events.is_empty() { for (l, _) in new_events.iter().take(3) { let id = m.find(edi_samn::Kind::Event, l).unwrap(); m.recall(&[(id, 1.0)], Some(&emb_avg)); } }
            }
        }
        for (l, _) in new_events.iter().filter(|(l, _)| !l.starts_with("PROC:") || tk < 2000) { say(&sh, now(), l.clone()); }

        {
            let mut r = sh.spikes.lock().unwrap();
            r.push_back((tk + 1, tick_spikes));
            while r.len() > 40 { r.pop_front(); }
        }
        vital.age_ticks += 1;
        ticks_run += 1;
        let (nn, ne) = sh.samn.lock().unwrap().stats();
        mood.tick(tk, vital.asleep, vital.energy, vital.sleep_pressure, cpu, nn);
        *sh.face.lock().unwrap() = serde_json::to_value(&mood.face).unwrap();
        if let Some(rc) = reg_hist.back() {
            let mut a = sh.activity.lock().unwrap();
            a.push_back((tk, (0..8).map(|k| rc[k] as f32 / n_reg[k].max(1) as f32).collect()));
            if a.len() > 200 { a.pop_front(); }
        }
        *sh.state.lock().unwrap() = serde_json::json!({
            "name": "EDI", "age_bio_s": vital.age_ticks as f64 * TICK_BIO_MS / 1000.0, "births": vital.births,
            "asleep": vital.asleep, "energy": vital.energy, "sleep_pressure": vital.sleep_pressure,
            "spikes_last_tick": vital.spikes_last, "speed_x_realtime": speed, "psi": {"cpu": cpu, "memory": mem},
            "memory": {"nodes": nn, "edges": ne}, "neurons": c.n, "synapses_edges": c.meta.n_edges, "metrics": metrics, "face": mood.face, "regions": RN, "um_per_unit": um_per_unit,
        });
        if vital.age_ticks % SNAPSHOT_EVERY == 0 { let m = sh.samn.lock().unwrap(); save(&state, &brain, &vital, &m)?; }
        if vital.energy < 0.7 { std::thread::sleep(t0.elapsed().mul_f64(1.0 - vital.energy)); }
    }
    let m = sh.samn.lock().unwrap();
    save(&state, &brain, &vital, &m)?;
    drop(m);
    say(&sh, now(), "pausa".into());
    Ok(())
}

/// Complejidad de Lempel-Ziv (LZ76, Kaspar-Schuster).
fn lz76(s: &[u8]) -> usize {
    let n = s.len();
    if n < 2 { return n; }
    let (mut c, mut l, mut i, mut k, mut kmax) = (1usize, 1usize, 0usize, 1usize, 1usize);
    loop {
        if s[i + k - 1] == s[l + k - 1] {
            k += 1;
            if l + k > n { c += 1; break; }
        } else {
            kmax = kmax.max(k);
            i += 1;
            if i == l { c += 1; l += kmax; if l + 1 > n { break; } i = 0; k = 1; kmax = 1; } else { k = 1; }
        }
    }
    c
}
