//! Motor LIF del conectoma de Drosophila para EDI.
//!
//! Ecuaciones de Shiu et al. 2024 (Brian2): dv/dt = (v0 - v + g)/tm, dg/dt = -g/tau,
//! integradas de forma exacta en cada paso. Sinapsis con retardo fijo; peso =
//! nº de sinapsis con signo × w_syn. Modo estocástico: cada sinapsis libera con
//! probabilidad `p_release` (binomial), con azar sembrado desde `entropy`.

pub mod entropy;

use anyhow::{Context, Result};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Binomial, Distribution};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
pub struct Meta {
    pub n: usize,
    pub n_edges: usize,
    pub groups: HashMap<String, Vec<u32>>,
}

/// Anatomía inmutable en formato CSR (presináptica → postsinápticas).
pub struct Connectome {
    pub n: usize,
    pub row_ptr: Vec<u32>,
    pub col: Vec<u32>,
    pub w: Vec<i32>,
    pub meta: Meta,
}

fn read_vec<T: Copy + Default>(path: &Path) -> Result<Vec<T>> {
    let bytes = std::fs::read(path).with_context(|| format!("leyendo {}", path.display()))?;
    let sz = std::mem::size_of::<T>();
    let mut out = vec![T::default(); bytes.len() / sz];
    // SAFETY: T es un entero plano (u32/i32/f32/i64) y los tamaños coinciden.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.as_mut_ptr() as *mut u8, out.len() * sz);
    }
    Ok(out)
}

impl Connectome {
    pub fn load(dir: &Path) -> Result<Self> {
        let meta: Meta = serde_json::from_str(&std::fs::read_to_string(dir.join("meta.json"))?)?;
        let c = Connectome {
            n: meta.n,
            row_ptr: read_vec(&dir.join("row_ptr.u32"))?,
            col: read_vec(&dir.join("col.u32"))?,
            w: read_vec(&dir.join("w.i32"))?,
            meta,
        };
        anyhow::ensure!(c.row_ptr.len() == c.n + 1 && c.col.len() == c.meta.n_edges, "conectoma corrupto");
        Ok(c)
    }

    pub fn group(&self, name: &str) -> &[u32] {
        self.meta.groups.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Control: permuta los destinos de las aristas. Conserva el grado de salida
    /// de cada presináptica y el de entrada de cada postsináptica, y los pesos.
    pub fn shuffled(&self, seed: u64) -> Self {
        use rand::seq::SliceRandom;
        use rand::SeedableRng;
        let mut col = self.col.clone();
        col.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
        Connectome {
            n: self.n,
            row_ptr: self.row_ptr.clone(),
            col,
            w: self.w.clone(),
            meta: Meta { n: self.n, n_edges: self.meta.n_edges, groups: self.meta.groups.clone() },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Params {
    pub dt: f64,     // ms
    pub v0: f64,     // mV reposo
    pub v_rst: f64,  // mV reset
    pub v_th: f64,   // mV umbral
    pub t_mbr: f64,  // ms
    pub tau: f64,    // ms
    pub t_rfc: f64,  // ms
    pub t_dly: f64,  // ms
    pub w_syn: f64,  // mV por sinapsis
    pub f_poi: f64,  // escala de la entrada Poisson
    /// None = determinista (como Shiu). Some(p) = liberación estocástica con prob. p,
    /// con el peso escalado por 1/p para conservar la media.
    pub p_release: Option<f64>,
}

impl Default for Params {
    fn default() -> Self {
        Params { dt: 0.1, v0: -52.0, v_rst: -52.0, v_th: -45.0, t_mbr: 20.0, tau: 5.0,
                 t_rfc: 2.2, t_dly: 1.8, w_syn: 0.275, f_poi: 250.0, p_release: None }
    }
}

/// Estado dinámico de un cerebro (separado de la anatomía).
pub struct Brain<'c> {
    pub c: &'c Connectome,
    pub p: Params,
    pub v: Vec<f32>,
    pub g: Vec<f32>,
    refr: Vec<u32>,
    ring: Vec<Vec<f32>>,
    step: u64,
    pub spikes: Vec<u32>,
    pub counts: Vec<u32>,
    // coeficientes de integración exacta
    eg: f32,
    ev: f32,
    kvg: f32,
    rfc_steps: u32,
    /// Motor por eventos: solo se calculan las neuronas fuera del reposo (como el cerebro,
    /// donde solo un 1–2 % está activo). Una neurona en reposo exacto (v=v0, g=0) no cambia,
    /// así que saltarla es exacto; vuelve a la lista cuando recibe entrada.
    pub event_driven: bool,
    active: Vec<u32>,
    is_active: Vec<bool>,
    ring_s: Vec<Vec<(u32, f32)>>,
    /// Neuronas actualizadas en el último paso (coste real de cómputo).
    pub last_updates: usize,
    /// Umbral de reposo (mV): por debajo, la neurona se fija en (v0, 0) y sale de la lista activa.
    pub eps: f32,
    /// Ganancia plástica por arista (estado aprendido, separado de la anatomía inmutable).
    pub edge_gain: Option<Vec<f32>>,
    /// Depresión sináptica a corto plazo (Tsodyks–Markram, presináptica): (U, τ_rec ms).
    /// Hipótesis de parámetros; evita la ignición epiléptica del LIF puro. Cálculo perezoso:
    /// solo se actualiza cuando la neurona dispara (coste ~0).
    pub std_dep: Option<(f32, f32)>,
    res: Vec<f32>,
    last_spike_step: Vec<u64>,
    /// Adaptación de frecuencia por umbral (Δθ mV por spike, τ ms). Hipótesis de parámetros.
    pub sfa: Option<(f32, f32)>,
    /// Desplazamiento de umbral por neurona (plasticidad homeostática intrínseca; estado aprendido).
    pub thr_offset: Option<Vec<f32>>,
    /// Atención multirresolución: solo se simulan las neuronas con máscara=true (foco); el resto no se calcula.
    pub focus: Option<Vec<bool>>,
    theta: Vec<f32>,
    theta_step: Vec<u64>,
}



impl<'c> Brain<'c> {
    pub fn new(c: &'c Connectome, p: Params) -> Self {
        let d = ((p.t_dly / p.dt).round() as usize).max(1);
        let (eg, ev) = ((-p.dt / p.tau).exp(), (-p.dt / p.t_mbr).exp());
        let kvg = p.tau / (p.tau - p.t_mbr) * (eg - ev);
        Brain {
            c, v: vec![p.v0 as f32; c.n], g: vec![0.0; c.n], refr: vec![0; c.n],
            ring: vec![vec![0.0; c.n]; d + 1], step: 0, spikes: Vec::new(), counts: vec![0; c.n],
            eg: eg as f32, ev: ev as f32, kvg: kvg as f32,
            rfc_steps: (p.t_rfc / p.dt).round() as u32, p,
            event_driven: true, active: Vec::new(), is_active: vec![false; c.n], ring_s: vec![Vec::new(); d + 1], last_updates: 0, eps: 1e-2, edge_gain: None, std_dep: None, res: vec![1.0; c.n], last_spike_step: vec![0; c.n], sfa: None, thr_offset: None, focus: None, theta: vec![0.0; c.n], theta_step: vec![0; c.n],
        }
    }

    #[inline]
    fn activate(&mut self, i: usize) {
        if let Some(f) = &self.focus { if !f[i] { return; } }
        if !self.is_active[i] {
            self.is_active[i] = true;
            self.active.push(i as u32);
        }
    }

    /// Fracción de neuronas activas ahora mismo.
    pub fn active_fraction(&self) -> f32 {
        if self.event_driven { self.active.len() as f32 / self.c.n as f32 } else { 1.0 }
    }

    /// Paso por eventos (mismo orden de operaciones que el denso: entrega → integración → umbral →
    /// propagación → Poisson → reset).
    fn step_event(&mut self, drive: &[(u32, f32)], rng: &mut ChaCha8Rng) {
        let (v0, vth) = (self.p.v0 as f32, self.p.v_th as f32);
        let (eg, ev, kvg) = (self.eg, self.ev, self.kvg);
        let d = self.ring_s.len();
        let slot = (self.step as usize) % d;
        let arriving = std::mem::take(&mut self.ring_s[slot]);
        for &(i, x) in &arriving {
            self.g[i as usize] += x;
            self.activate(i as usize);
        }
        let mut buf = arriving;
        buf.clear();
        self.spikes.clear();
        self.last_updates = self.active.len();
        let mut k = 0;
        while k < self.active.len() {
            let i = self.active[k] as usize;
            if self.refr[i] > 0 {
                self.refr[i] -= 1;
                k += 1;
                continue;
            }
            let (u, g) = (self.v[i] - v0, self.g[i]);
            self.v[i] = v0 + u * ev + g * kvg;
            self.g[i] = g * eg;
            let th = if let Some((_, tau)) = self.sfa {
                if self.theta[i] > 0.0 {
                    let dts = (self.step - self.theta_step[i]) as f32 * self.p.dt as f32;
                    self.theta[i] *= (-dts / tau).exp();
                    self.theta_step[i] = self.step;
                    if self.theta[i] < 1e-3 { self.theta[i] = 0.0; }
                }
                self.theta[i]
            } else { 0.0 };
            let off = self.thr_offset.as_ref().map(|o| o[i]).unwrap_or(0.0);
            if self.v[i] > vth + th + off {
                self.spikes.push(i as u32);
            } else if (self.v[i] - v0).abs() < self.eps && self.g[i].abs() < self.eps {
                self.v[i] = v0;
                self.g[i] = 0.0;
                self.is_active[i] = false;
                self.active.swap_remove(k);
                continue;
            }
            k += 1;
        }
        self.spikes.sort_unstable();
        let w_syn = self.p.w_syn as f32;
        for si in 0..self.spikes.len() {
            let s = self.spikes[si] as usize;
            let mut eff_std = 1.0f32;
            if let Some((u, tau)) = self.std_dep {
                let dt_ms = (self.step - self.last_spike_step[s]) as f32 * self.p.dt as f32;
                let x = 1.0 - (1.0 - self.res[s]) * (-dt_ms / tau).exp();
                eff_std = x;               // eficacia relativa a recursos llenos (U·x / U·1)
                self.res[s] = x - u * x;   // se consume la fracción U
                self.last_spike_step[s] = self.step;
            }
            for e in self.c.row_ptr[s] as usize..self.c.row_ptr[s + 1] as usize {
                let kk = self.c.w[e];
                let eff = match self.p.p_release {
                    None => kk as f32,
                    Some(pr) => {
                        let n = kk.unsigned_abs() as u64;
                        let rel = Binomial::new(n, pr).map(|b| b.sample(rng)).unwrap_or(0);
                        kk.signum() as f32 * rel as f32 / pr as f32
                    }
                };
                let gain = self.edge_gain.as_ref().map(|g| g[e]).unwrap_or(1.0);
                buf.push((self.c.col[e], eff * w_syn * gain * eff_std));
            }
        }
        self.ring_s[slot] = buf;
        let jump = (self.p.w_syn * self.p.f_poi) as f32;
        let dt_s = (self.p.dt / 1000.0) as f32;
        for &(i, hz) in drive {
            if rng.gen::<f32>() < hz * dt_s {
                self.v[i as usize] += jump;
                self.activate(i as usize);
            }
        }
        for si in 0..self.spikes.len() {
            let s = self.spikes[si] as usize;
            self.v[s] = self.p.v_rst as f32;
            self.g[s] = 0.0;
            self.refr[s] = self.rfc_steps;
            self.counts[s] += 1;
            if let Some((d, _)) = self.sfa { self.theta[s] += d; self.theta_step[s] = self.step; }
        }
        self.step += 1;
    }

    pub fn reset_counts(&mut self) {
        self.counts.iter_mut().for_each(|x| *x = 0);
    }

    /// Avanza un paso dt. `drive`: (neurona, tasa Hz) con entrada Poisson sobre v.
    pub fn step(&mut self, drive: &[(u32, f32)], rng: &mut ChaCha8Rng) {
        if self.event_driven {
            return self.step_event(drive, rng);
        }
        self.last_updates = self.c.n;
        let (v0, vth) = (self.p.v0 as f32, self.p.v_th as f32);
        let (eg, ev, kvg) = (self.eg, self.ev, self.kvg);
        let d = self.ring.len();
        let slot = (self.step as usize) % d;
        // 1) entrega de lo que llega + integración exacta (congelada en refractario) + umbral
        self.spikes.clear();
        {
            let arriving = &mut self.ring[slot];
            for i in 0..self.c.n {
                let x = arriving[i];
                if x != 0.0 {
                    self.g[i] += x;
                    arriving[i] = 0.0;
                }
                if self.refr[i] > 0 {
                    self.refr[i] -= 1;
                    continue;
                }
                let (u, g) = (self.v[i] - v0, self.g[i]);
                self.v[i] = v0 + u * ev + g * kvg;
                self.g[i] = g * eg;
                if self.v[i] > vth {
                    self.spikes.push(i as u32);
                }
            }
        }
        // 2) propagación: llega al inicio del paso t+D+1 (≡ final de t+D, como Brian2)
        let w_syn = self.p.w_syn as f32;
        let tgt = slot;
        for &s in &self.spikes {
            let (a, b) = (self.c.row_ptr[s as usize] as usize, self.c.row_ptr[s as usize + 1] as usize);
            for e in a..b {
                let k = self.c.w[e];
                let eff = match self.p.p_release {
                    None => k as f32,
                    Some(pr) => {
                        let n = k.unsigned_abs() as u64;
                        let rel = Binomial::new(n, pr).map(|b| b.sample(rng)).unwrap_or(0);
                        k.signum() as f32 * rel as f32 / pr as f32
                    }
                };
                self.ring[tgt][self.c.col[e] as usize] += eff * w_syn;
            }
        }
        // 4) entrada Poisson (Shiu: PoissonInput sobre v)
        let jump = (self.p.w_syn * self.p.f_poi) as f32;
        let dt_s = (self.p.dt / 1000.0) as f32;
        for &(i, hz) in drive {
            if rng.gen::<f32>() < hz * dt_s {
                self.v[i as usize] += jump;
            }
        }
        // 5) reset
        for &s in &self.spikes {
            let s = s as usize;
            self.v[s] = self.p.v_rst as f32;
            self.g[s] = 0.0;
            self.refr[s] = self.rfc_steps;
            self.counts[s] += 1;
        }
        self.step += 1;
    }

    pub fn run(&mut self, ms: f64, drive: &[(u32, f32)], rng: &mut ChaCha8Rng) {
        for _ in 0..(ms / self.p.dt).round() as usize {
            self.step(drive, rng);
        }
    }
}
