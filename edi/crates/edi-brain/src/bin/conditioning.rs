//! Prueba 2 — aprendizaje dopaminérgico real (condicionamiento olfativo, como en la mosca).
//!
//! Olor A = ORN de DA1+VA1d+DL3; olor B = ORN de VA1v+VL1+VM4 (glomérulos reales).
//! Entrenamiento: A emparejado con activación de neuronas de dopamina (optogenética in silico).
//! Regla de tres factores en Kenyon→MBON: Δg_e = −η · actividad_KC(pre) · dopamina(MBON post),
//! donde la dopamina de cada MBON = spikes de las DAN que sinapsan sobre ella en el conectoma.
//! Medida: respuesta de las MBON a A y B antes/después. Memoria = depresión específica de A.
//! Condiciones: conectoma real, barajado y real sin plasticidad (control).
//! Uso: conditioning <dir_conectoma> <out.json> [dan: pam|ppl1|all]

use edi_brain::{Brain, Connectome, Params};
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, HashSet};

const ODOR_MS: f64 = 300.0;
const REST_MS: f64 = 1500.0;
static mut ORN_HZ: f32 = 120.0;
const DAN_HZ: f32 = 150.0;
const ETA: f32 = 0.0005;
const TRIALS_TRAIN: usize = 6;
const TRIALS_TEST: usize = 4;

struct Setup {
    a: Vec<(u32, f32)>,
    b: Vec<(u32, f32)>,
    dan: Vec<(u32, f32)>,
    kc: Vec<u32>,
    mbon: Vec<u32>,
    /// aristas plásticas KC→MBON: (arista, kc, mbon)
    plastic: Vec<(usize, u32, u32)>,
    /// DANs que inervan cada MBON (según el conectoma de esta condición)
    dan_of: HashMap<u32, Vec<u32>>,
}

fn setup(c: &Connectome, dan_kind: &str) -> Setup {
    let g = |n: &str| c.group(n).to_vec();
    let drive = |names: &[&str], hz: f32| names.iter().flat_map(|n| g(n)).map(|i| (i, hz)).collect::<Vec<_>>();
    let dans: Vec<u32> = match dan_kind { "pam" => g("dan_pam"), "ppl1" => g("dan_ppl1"), _ => g("dan") };
    let kc = g("kenyon_cell");
    let mbon = g("mbon");
    let mset: HashSet<u32> = mbon.iter().cloned().collect();
    let dset: HashSet<u32> = dans.iter().cloned().collect();
    let mut plastic = vec![];
    for &k in &kc {
        for e in c.row_ptr[k as usize] as usize..c.row_ptr[k as usize + 1] as usize {
            if mset.contains(&c.col[e]) { plastic.push((e, k, c.col[e])); }
        }
    }
    let mut dan_of: HashMap<u32, Vec<u32>> = HashMap::new();
    for &d in &dans {
        for e in c.row_ptr[d as usize] as usize..c.row_ptr[d as usize + 1] as usize {
            if mset.contains(&c.col[e]) { dan_of.entry(c.col[e]).or_default().push(d); }
        }
    }
    let _ = dset;
    let hz = unsafe { ORN_HZ };
    Setup { a: drive(&["orn_da1", "orn_va1d", "orn_dl3"], hz), b: drive(&["orn_va1v", "orn_vl1", "orn_vm4"], hz),
            dan: dans.iter().map(|&i| (i, DAN_HZ)).collect(), kc, mbon, plastic, dan_of }
}

/// Hipótesis (opcional, EDI_NO_KCKC=1): silenciar sinapsis Kenyon→Kenyon (axo-axónicas, función incierta),
/// que en un LIF puramente excitatorio provocan excitación descontrolada del cuerpo fungiforme.
fn std_of() -> Option<(f32, f32)> { let u: f32 = std::env::var("EDI_STD_U").ok().and_then(|x| x.parse().ok()).unwrap_or(0.5); std::env::var("EDI_STD").ok().and_then(|t| t.parse().ok()).map(|t| (u, t)) }

fn sfa_of() -> Option<(f32, f32)> { std::env::var("EDI_SFA").ok().and_then(|t| t.parse().ok()).map(|d| (d, 200.0)) }

fn gains(c: &Connectome) -> Vec<f32> {
    let mut g = vec![1.0f32; c.meta.n_edges];
    // La dopamina es neuromoduladora (receptores acoplados a proteína G), no excitación rápida:
    // sus sinapsis no transmiten corriente; solo actúan en la regla de plasticidad (tercer factor).
    if std::env::var("EDI_DA_MOD").as_deref() != Ok("0") {
        for &d in c.group("dan") { for e in c.row_ptr[d as usize] as usize..c.row_ptr[d as usize + 1] as usize { g[e] = 0.0; } }
    }
    if std::env::var("EDI_NO_KCKC").as_deref() == Ok("1") {
        let kc: HashSet<u32> = c.group("kenyon_cell").iter().cloned().collect();
        for &k in &kc { for e in c.row_ptr[k as usize] as usize..c.row_ptr[k as usize + 1] as usize { if kc.contains(&c.col[e]) { g[e] = 0.0; } } }
    }
    g
}

/// Calibración homeostática de umbrales de KC (misma regla que `homeostasis`), por conectoma.
fn calibrate(c: &Connectome, rounds: usize) -> Vec<f32> {
    let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(std::env::var("EDI_BRAIN_DIR").unwrap() + "/meta.json").unwrap()).unwrap();
    let types: Vec<String> = meta["orn_types"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).filter(|t| !c.group(t).is_empty()).collect();
    let kc = c.group("kenyon_cell").to_vec();
    let mut off = vec![0f32; c.n];
    let mut b = Brain::new(c, Params { dt: 0.25, ..Params::default() });
    b.edge_gain = Some(gains(c)); b.std_dep = std_of(); b.sfa = sfa_of();
    let mut rng = ChaCha8Rng::seed_from_u64(9);
    for _ in 0..rounds {
        let mut t = types.clone(); t.shuffle(&mut rng);
        let od: Vec<(u32, f32)> = t[..3].iter().flat_map(|x| c.group(x).to_vec()).map(|i| (i, 60.0)).collect();
        b.thr_offset = Some(off.clone());
        b.reset_counts(); b.run(300.0, &od, &mut rng);
        let fired: Vec<bool> = kc.iter().map(|&k| b.counts[k as usize] > 0).collect();
        b.run(1000.0, &[], &mut rng);
        for (j, &k) in kc.iter().enumerate() { off[k as usize] = (off[k as usize] + 1.0 * (if fired[j] { 0.95 } else { -0.05 })).clamp(-8.0, 40.0); }
    }
    off
}

fn trial(b: &mut Brain, drive: &[(u32, f32)], rng: &mut ChaCha8Rng) -> Vec<u32> {
    b.reset_counts();
    b.run(ODOR_MS, drive, rng);
    let counts = b.counts.clone();
    b.run(REST_MS, &[], rng);
    counts
}

fn mbon_resp(cnt: &[u32], s: &Setup) -> f64 { s.mbon.iter().map(|&m| cnt[m as usize] as f64).sum() }

/// Corriente sináptica KC→MBON (lo que cambia el aprendizaje): Σ spikes_KC · |w| · ganancia.
fn mbon_drive(cnt: &[u32], s: &Setup, c: &Connectome, gain: &[f32]) -> f64 {
    s.plastic.iter().map(|&(e, k, _)| cnt[k as usize] as f64 * c.w[e].unsigned_abs() as f64 * gain[e] as f64).sum()
}

fn run(c: &Connectome, s: &Setup, learn: bool, seed: u64, off: &Option<Vec<f32>>) -> serde_json::Value {
    let mut b = Brain::new(c, Params { dt: 0.25, ..Params::default() });
    b.thr_offset = off.clone();
    b.edge_gain = Some(gains(c));
    b.std_dep = std_of();
    b.sfa = sfa_of();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let test = |b: &mut Brain, rng: &mut ChaCha8Rng| {
        let (mut ra, mut rb, mut kca, mut kcb) = (0.0, 0.0, 0usize, 0usize);
        for _ in 0..TRIALS_TEST {
            let ca = trial(b, &s.a, rng); ra += mbon_drive(&ca, s, c, b.edge_gain.as_ref().unwrap()); kca += s.kc.iter().filter(|&&k| ca[k as usize] > 0).count();
            let cb = trial(b, &s.b, rng); rb += mbon_drive(&cb, s, c, b.edge_gain.as_ref().unwrap()); kcb += s.kc.iter().filter(|&&k| cb[k as usize] > 0).count();
        }
        let n = TRIALS_TEST as f64;
        (ra / n, rb / n, kca as f64 / n, kcb as f64 / n)
    };
    let (a0, b0, kca, kcb) = test(&mut b, &mut rng);
    let mut dan_spikes = 0u64;
    for _ in 0..TRIALS_TRAIN {
        // A + dopamina
        let mut drive = s.a.clone();
        drive.extend(s.dan.iter().cloned());
        let cnt = trial(&mut b, &drive, &mut rng);
        dan_spikes += s.dan.iter().map(|(d, _)| cnt[*d as usize] as u64).sum::<u64>();
        if learn {
            let gain = b.edge_gain.as_mut().unwrap();
            for &(e, k, m) in &s.plastic {
                let pre = cnt[k as usize] as f32;
                if pre == 0.0 { continue; }
                let dop: f32 = s.dan_of.get(&m).map(|v| v.iter().map(|&d| cnt[d as usize] as f32).sum()).unwrap_or(0.0);
                gain[e] = (gain[e] * (1.0 - ETA * pre * dop).max(0.0)).max(0.0);
            }
        }
        // B sin dopamina (control dentro del animal)
        trial(&mut b, &s.b, &mut rng);
    }
    let (a1, b1, _, _) = test(&mut b, &mut rng);
    let g = b.edge_gain.as_ref().unwrap();
    let depressed = s.plastic.iter().filter(|(e, _, _)| g[*e] < 0.99).count();
    let ra = if a0 > 0.0 { a1 / a0 } else { f64::NAN };
    let rb = if b0 > 0.0 { b1 / b0 } else { f64::NAN };
    serde_json::json!({"mbon_A_antes": a0, "mbon_A_despues": a1, "mbon_B_antes": b0, "mbon_B_despues": b1,
        "ratio_A": ra, "ratio_B": rb, "indice_memoria": rb - ra,
        "kc_activas_A": kca, "kc_activas_B": kcb, "spikes_dopamina_entreno": dan_spikes,
        "aristas_plasticas": s.plastic.len(), "mbon_con_dopamina": s.dan_of.len(), "aristas_deprimidas": depressed})
}

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let c = Connectome::load(std::path::Path::new(&a[1]))?;
    let kind = a.get(3).map(|s| s.as_str()).unwrap_or("all");
    if let Some(h) = a.get(4).and_then(|x| x.parse::<f32>().ok()) { unsafe { ORN_HZ = h; } }
    if a.get(5).map(|s| s == "sweep").unwrap_or(false) {
        for hz in [60.0f32, 150.0] {
            unsafe { ORN_HZ = hz; }
            let s = setup(&c, kind);
            let mut b = Brain::new(&c, Params { dt: 0.25, ..Params::default() });
            b.edge_gain = Some(gains(&c));
            b.std_dep = std_of();
    b.sfa = sfa_of();
            let mut rng = ChaCha8Rng::seed_from_u64(1);
            let ca = trial(&mut b, &s.a, &mut rng);
            let mut b = Brain::new(&c, Params { dt: 0.25, ..Params::default() });
            b.edge_gain = Some(gains(&c));
            b.std_dep = std_of();
    b.sfa = sfa_of();
            let cb = trial(&mut b, &s.b, &mut rng);
            let ka: HashSet<u32> = s.kc.iter().cloned().filter(|&k| ca[k as usize] > 0).collect();
            let kb: HashSet<u32> = s.kc.iter().cloned().filter(|&k| cb[k as usize] > 0).collect();
            let inter = ka.intersection(&kb).count();
            println!("ORN {hz:>4} Hz · KC activas A {} ({:.1} %) · B {} · solapamiento {} · MBON A {:.0} B {:.0}", ka.len(),
                100.0 * ka.len() as f64 / s.kc.len() as f64, kb.len(), inter, mbon_resp(&ca, &s), mbon_resp(&cb, &s));
        }
        return Ok(());
    }
    let sh = c.shuffled(101);
    let (sr, ss) = (setup(&c, kind), setup(&sh, kind));
    let homeo = std::env::var("EDI_HOMEO").as_deref() == Ok("1");
    let (or_, os_) = if homeo { (Some(calibrate(&c, 100)), Some(calibrate(&sh, 100))) } else { (None, None) };
    let mut out = serde_json::json!({"dan": kind, "eta": ETA, "trials_train": TRIALS_TRAIN, "homeostasis": homeo,
        "std": std_of().map(|x| vec![x.0, x.1]), "sfa": sfa_of().map(|x| vec![x.0, x.1]), "sin_kc_kc": std::env::var("EDI_NO_KCKC").as_deref() == Ok("1"), "dopamina_moduladora": std::env::var("EDI_DA_MOD").as_deref() != Ok("0")});
    for seed in [1u64, 2, 3] {
        out[format!("real_s{seed}")] = run(&c, &sr, true, seed, &or_);
        out[format!("real_sin_plasticidad_s{seed}")] = run(&c, &sr, false, seed, &or_);
        out[format!("barajado_s{seed}")] = run(&sh, &ss, true, seed, &os_);
        eprintln!("semilla {seed} lista");
    }
    std::fs::write(&a[2], serde_json::to_string_pretty(&out)?)?;
    for (k, v) in out.as_object().unwrap() {
        if let Some(o) = v.as_object() {
            println!("{k:28} corriente MBON A {:.0}→{:.0} · B {:.0}→{:.0} · memoria {:+.3} · KC A/B {:.0}/{:.0} · deprimidas {}",
                o["mbon_A_antes"].as_f64().unwrap(), o["mbon_A_despues"].as_f64().unwrap(), o["mbon_B_antes"].as_f64().unwrap(),
                o["mbon_B_despues"].as_f64().unwrap(), o["indice_memoria"].as_f64().unwrap_or(f64::NAN),
                o["kc_activas_A"].as_f64().unwrap(), o["kc_activas_B"].as_f64().unwrap(), o["aristas_deprimidas"]);
        }
    }
    Ok(())
}
