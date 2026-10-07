//! Lo probado en experiments/learning y experiments/efficiency, ahora vivo en EDI:
//! - cada concepto (palabra, ventana, puerto, proceso) le llega como un "olor": 3 glomérulos
//!   reales (de 53 tipos de ORN) elegidos por hash → código disperso en las células de Kenyon;
//! - la dopamina es moduladora: tus reacciones (bien/mal, /reward) activan DANs PAM (premio)
//!   o PPL1 (castigo) y deprimen las sinapsis KC→MBON con traza de elegibilidad (3 factores);
//! - la valencia aprendida de cada concepto se lee de sus sinapsis KC→MBON (no se inventa).
#![allow(dead_code)]
use edi_brain::Connectome;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub fn orn_hz() -> f32 { std::env::var("EDI_ORN_HZ").ok().and_then(|x| x.parse().ok()).unwrap_or(120.0) }
pub const DAN_HZ: f32 = 150.0;
fn eta() -> f32 { std::env::var("EDI_ETA").ok().and_then(|x| x.parse().ok()).unwrap_or(0.004) }
fn trace_decay() -> f32 { std::env::var("EDI_TRACE").ok().and_then(|x| x.parse().ok()).unwrap_or(0.8) }

pub struct Mb {
    types: Vec<Vec<u32>>,
    kc: Vec<u32>,
    pub pam: Vec<u32>,
    pub ppl1: Vec<u32>,
    /// aristas plásticas KC→MBON: (arista, índice de KC en `kc`, mbon)
    plastic: Vec<(usize, u32, u32)>,
    dan_of: HashMap<u32, Vec<u32>>,
    pam_set: HashSet<u32>,
    ppl1_set: HashSet<u32>,
    trace: Vec<f32>,
    /// actividad media de cada KC entre conceptos (lo común se descuenta, como la inhibición de APL)
    kc_mean: Vec<f32>,
    n_seen: f32,
}

/// Valencia aprendida por concepto, leída de las sinapsis que su olor activa (rastro de memoria).
#[derive(Default, Serialize, Deserialize)]
pub struct Valence { pub now: HashMap<String, f32> }

impl Valence {
    /// >0 le gusta (KC→MBON de PAM deprimidas por premio), <0 le disgusta (las de PPL1, por castigo).
    /// Relativa a la mediana de todos sus conceptos (la mayoría neutros), que absorbe el sesgo común.
    pub fn of(&self, k: &str) -> Option<f32> {
        let v = *self.now.get(k)?;
        let mut all: Vec<f32> = self.now.values().copied().collect();
        if all.len() < 3 { return Some(0.0); }
        all.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Some(v - all[all.len() / 2])
    }
}

pub fn fnv(s: &str) -> u64 { s.bytes().fold(0xcbf29ce484222325, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3)) }

impl Mb {
    pub fn new(c: &Connectome) -> Self {
        let meta: Vec<String> = c.meta.groups.keys().filter(|k| k.starts_with("orn_")).cloned().collect();
        let mut types: Vec<Vec<u32>> = meta.iter().map(|t| c.group(t).to_vec()).filter(|v| !v.is_empty()).collect();
        types.sort();
        let kc = c.group("kenyon_cell").to_vec();
        let kidx: HashMap<u32, u32> = kc.iter().enumerate().map(|(j, &k)| (k, j as u32)).collect();
        let mset: HashSet<u32> = c.group("mbon").iter().cloned().collect();
        let mut plastic = vec![];
        for &k in &kc {
            for e in c.row_ptr[k as usize] as usize..c.row_ptr[k as usize + 1] as usize {
                if mset.contains(&c.col[e]) { plastic.push((e, kidx[&k], c.col[e])); }
            }
        }
        let (pam, ppl1) = (c.group("dan_pam").to_vec(), c.group("dan_ppl1").to_vec());
        // cada MBON pertenece al compartimento de su entrada dopaminérgica dominante (nº de sinapsis)
        let mut dan_of: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut wsum: HashMap<u32, (i64, i64)> = HashMap::new();
        for (k, ds) in [(0, &pam), (1, &ppl1)] {
            for &d in ds.iter() {
                for e in c.row_ptr[d as usize] as usize..c.row_ptr[d as usize + 1] as usize {
                    if mset.contains(&c.col[e]) {
                        let w = wsum.entry(c.col[e]).or_default();
                        if k == 0 { w.0 += c.w[e].abs() as i64 } else { w.1 += c.w[e].abs() as i64 }
                    }
                }
            }
        }
        let (mut mp, mut mq) = (HashSet::new(), HashSet::new());
        for (&m, &(a, b)) in &wsum { if a >= b { mp.insert(m); } else { mq.insert(m); } }
        for (ds, set) in [(&pam, &mp), (&ppl1, &mq)] {
            for &d in ds.iter() {
                for e in c.row_ptr[d as usize] as usize..c.row_ptr[d as usize + 1] as usize {
                    if set.contains(&c.col[e]) { dan_of.entry(c.col[e]).or_default().push(d); }
                }
            }
        }
        for v in dan_of.values_mut() { v.sort(); v.dedup(); }
        let n = kc.len();
        Mb { types, kc, pam, ppl1, plastic, dan_of, pam_set: mp, ppl1_set: mq, trace: vec![0.0; n], kc_mean: vec![0.0; n], n_seen: 0.0 }
    }

    /// Ganancias iniciales: la dopamina no transmite corriente (moduladora).
    pub fn gains(&self, c: &Connectome) -> Vec<f32> {
        let mut g = vec![1.0f32; c.meta.n_edges];
        for &d in c.group("dan") { for e in c.row_ptr[d as usize] as usize..c.row_ptr[d as usize + 1] as usize { g[e] = 0.0; } }
        g
    }

    /// Olor del concepto: 3 tipos de ORN por hash.
    pub fn odor(&self, label: &str) -> Vec<(u32, f32)> {
        let hz = orn_hz();
        let h = fnv(label);
        let n = self.types.len() as u64;
        let mut pick: Vec<usize> = vec![];
        let mut k = 0u64;
        while pick.len() < 3 && n >= 3 { let t = ((h.rotate_left(17 * k as u32) ^ k.wrapping_mul(0x9e3779b97f4a7c15)) % n) as usize; if !pick.contains(&t) { pick.push(t); } k += 1; }
        pick.iter().flat_map(|&t| self.types[t].iter().map(|&i| (i, hz))).collect()
    }

    /// Valencia desde las sinapsis: depresión media (ponderada por spikes de KC) hacia MBON de PAM
    /// menos hacia MBON de PPL1.
    pub fn valence(&self, kc_counts: &[f32], gain: &[f32]) -> f32 { let (p, q) = self.valence2(kc_counts, gain); p - q }
    /// Registra el patrón de KC de un concepto en la media (llamar tras medir su valencia).
    pub fn seen(&mut self, kc_counts: &[f32]) {
        self.n_seen += 1.0;
        let a = (1.0 / self.n_seen).max(0.02);
        for (m, &x) in self.kc_mean.iter_mut().zip(kc_counts) { *m += a * (x - *m); }
    }
    pub fn valence2(&self, kc_counts: &[f32], gain: &[f32]) -> (f32, f32) {
        let spec: Vec<f32> = kc_counts.iter().zip(&self.kc_mean).map(|(&x, &m)| (x - m).max(0.0)).collect();
        let kc_counts = &spec[..];
        let (mut dp, mut wp, mut dq, mut wq) = (0f32, 0f32, 0f32, 0f32);
        for &(e, j, m) in &self.plastic {
            let w = kc_counts[j as usize];
            if w == 0.0 { continue; }
            if self.pam_set.contains(&m) { dp += w * (1.0 - gain[e]); wp += w; }
            if self.ppl1_set.contains(&m) { dq += w * (1.0 - gain[e]); wq += w; }
        }
        // específica: menos la depresión media de todo el compartimento (lo inespecífico)
        let (mut gp, mut np, mut gq, mut nq) = (0f32, 0f32, 0f32, 0f32);
        for &(e, _, m) in &self.plastic {
            if self.pam_set.contains(&m) { gp += 1.0 - gain[e]; np += 1.0; }
            if self.ppl1_set.contains(&m) { gq += 1.0 - gain[e]; nq += 1.0; }
        }
        (dp / wp.max(1e-6) - gp / np.max(1.0), dq / wq.max(1e-6) - gq / nq.max(1.0))
    }
    pub fn n_dan_mbon(&self) -> (usize, usize, usize) { (self.pam_set.len(), self.ppl1_set.len(), self.pam_set.intersection(&self.ppl1_set).count()) }
    pub fn kc_counts(&self, counts: &[u32], acc: &mut Vec<f32>) {
        if acc.len() != self.kc.len() { *acc = vec![0.0; self.kc.len()]; }
        for (j, &k) in self.kc.iter().enumerate() { acc[j] += counts[k as usize] as f32; }
    }

    pub fn kc_active(&self, counts: &[u32]) -> usize { self.kc.iter().filter(|&&k| counts[k as usize] > 0).count() }
    pub fn n_kc(&self) -> usize { self.kc.len() }

    /// Regla de 3 factores tras cada tick: traza de KC × dopamina en el MBON → depresión KC→MBON.
    pub fn learn(&mut self, counts: &[u32], gain: &mut [f32]) -> usize {
        let td = trace_decay();
        for (j, &k) in self.kc.iter().enumerate() { self.trace[j] = self.trace[j] * td + counts[k as usize] as f32; }
        let mut changed = 0;
        let eta = eta();
        for &(e, j, m) in &self.plastic {
            let pre = self.trace[j as usize];
            if pre < 0.5 { continue; }
            // dopamina media por DAN que inerva este MBON (spikes en el tick)
            let dop: f32 = self.dan_of.get(&m).map(|v| v.iter().map(|&d| counts[d as usize] as f32).sum::<f32>() / v.len() as f32).unwrap_or(0.0);
            if dop > 0.0 { gain[e] = (gain[e] * (1.0 - eta * pre * dop).max(0.0)).max(0.0); changed += 1; }
        }
        changed
    }

    /// Solo las ganancias plásticas (para persistir lo aprendido).
    pub fn save_gains(&self, gain: &[f32]) -> Vec<u8> { self.plastic.iter().flat_map(|&(e, _, _)| gain[e].to_le_bytes()).collect() }
    pub fn load_gains(&self, raw: &[u8], gain: &mut [f32]) -> bool {
        if raw.len() != self.plastic.len() * 4 { return false; }
        for (k, &(e, _, _)) in self.plastic.iter().enumerate() { gain[e] = f32::from_le_bytes(raw[k * 4..k * 4 + 4].try_into().unwrap()); }
        true
    }
    pub fn depressed(&self, gain: &[f32]) -> usize { self.plastic.iter().filter(|&&(e, _, _)| gain[e] < 0.999).count() }
}
