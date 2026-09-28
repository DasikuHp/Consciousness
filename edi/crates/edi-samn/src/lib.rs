//! SAMN v0 — memoria asociativa semántico-episódica (docs/design/samn.md).
//!
//! Grafo dinámico: episodios → eventos → conceptos → habilidades, más palabras y entidades.
//! Plasticidad hebbiana + causal (ventana), fuerza de base tipo ACT-R, recuperación por
//! activación propagada con decaimiento e inhibición lateral (k-WTA por tipo), y sueño:
//! NREM consolida (eventos que se repiten entre episodios → concepto; secuencias → habilidad),
//! REM recombina (A→B y B→C ⇒ A→C marcada `Dreamed`) y se olvida por decaimiento.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const EMB: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind { Episode, Event, Percept, Concept, Skill, Word, Entity, SelfNode }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Src { Measured, Inferred, Dreamed }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rel { Temporal, Causal, Contains, Similar, Supports, Predicts, Grounds, Involves }

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub kind: Kind,
    pub label: String,
    pub a: f32,
    pub v: f32,
    pub c: f32,
    pub src: Src,
    pub emb: [f32; EMB],
    pub created: u64,
    pub accesses: Vec<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Edge { pub to: u32, pub w: f32, pub rel: Rel, pub last: u64, pub n: u32 }

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Params {
    pub eta: f32,        // hebbiano
    pub lambda: f32,     // olvido por paso de sueño
    pub d: f32,          // decaimiento ACT-R
    pub delta: f32,      // decaimiento de activación
    pub steps: usize,    // pasos de propagación
    pub k: usize,        // k-WTA por tipo
    pub tau: u64,        // ventana causal (ticks)
    pub prune: f32,      // umbral de poda
    pub min_support: usize, // episodios para consolidar
}
impl Default for Params {
    fn default() -> Self {
        Params { eta: 0.05, lambda: 0.02, d: 0.5, delta: 0.2, steps: 5, k: 7, tau: 40, prune: 0.02, min_support: 3 }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Samn {
    pub nodes: Vec<Node>,
    pub adj: Vec<Vec<Edge>>,
    pub t: u64,
    pub p: Params,
    #[serde(skip)]
    index: HashMap<(Kind, String), u32>,
    #[serde(skip)]
    cur_ep: Option<u32>,
    #[serde(skip)]
    last_ev: Option<(u32, u64)>,
    #[serde(skip)]
    ep_events: Vec<u32>,
    pub episodes: Vec<Vec<u32>>,
}

pub struct SleepReport { pub concepts: usize, pub skills: usize, pub dreamed: usize, pub pruned: usize }

fn cos(a: &[f32; EMB], b: &[f32; EMB]) -> f32 {
    let (mut d, mut x, mut y) = (0f32, 0f32, 0f32);
    for i in 0..EMB { d += a[i] * b[i]; x += a[i] * a[i]; y += b[i] * b[i]; }
    if x == 0.0 || y == 0.0 { 0.0 } else { d / (x.sqrt() * y.sqrt()) }
}

impl Samn {
    pub fn new() -> Self { Samn { p: Params::default(), ..Default::default() } }

    pub fn reindex(&mut self) {
        self.index = self.nodes.iter().enumerate().map(|(i, n)| ((n.kind, n.label.clone()), i as u32)).collect();
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_vec(self)?)?;
        std::fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let mut s: Samn = serde_json::from_slice(&std::fs::read(path)?)?;
        s.reindex();
        Ok(s)
    }

    /// Base-level (ACT-R): B = ln Σ (t - t_k)^-d
    pub fn base(&self, i: u32) -> f32 {
        let n = &self.nodes[i as usize];
        let s: f32 = n.accesses.iter().map(|&tk| ((self.t.saturating_sub(tk) + 1) as f32).powf(-self.p.d)).sum();
        if s > 0.0 { s.ln() } else { -5.0 }
    }

    /// Crea o refuerza un nodo.
    pub fn observe(&mut self, kind: Kind, label: &str, emb: Option<&[f32; EMB]>, v: f32, src: Src) -> u32 {
        let t = self.t;
        let id = match self.index.get(&(kind, label.to_string())) {
            Some(&i) => i,
            None => {
                let i = self.nodes.len() as u32;
                self.nodes.push(Node { kind, label: label.into(), a: 0.0, v: 0.0, c: 0.3, src, emb: [0.0; EMB], created: t, accesses: vec![] });
                self.adj.push(vec![]);
                self.index.insert((kind, label.to_string()), i);
                i
            }
        };
        let n = &mut self.nodes[id as usize];
        n.accesses.push(t);
        if n.accesses.len() > 64 { n.accesses.remove(0); }
        n.c = (n.c + 0.1).min(1.0);
        n.v = 0.8 * n.v + 0.2 * v;
        if src == Src::Measured { n.src = Src::Measured; }
        if let Some(e) = emb { for k in 0..EMB { n.emb[k] = 0.8 * n.emb[k] + 0.2 * e[k]; } }
        id
    }

    pub fn find(&self, kind: Kind, label: &str) -> Option<u32> { self.index.get(&(kind, label.to_string())).copied() }

    /// Arista con refuerzo hebbiano: w += eta·(1-w) (acotado en [0,1]).
    pub fn link(&mut self, a: u32, b: u32, rel: Rel, gain: f32) {
        let t = self.t;
        let eta = self.p.eta * gain;
        let list = &mut self.adj[a as usize];
        match list.iter_mut().find(|e| e.to == b && e.rel == rel) {
            Some(e) => { e.w += eta * (1.0 - e.w); e.last = t; e.n += 1; }
            None => list.push(Edge { to: b, w: (0.2 * gain).min(1.0), rel, last: t, n: 1 }),
        }
    }

    pub fn tick(&mut self) { self.t += 1; }

    // ---- episodios ----
    pub fn in_episode(&self) -> bool { self.cur_ep.is_some() }

    pub fn begin_episode(&mut self, label: &str, emb: &[f32; EMB]) -> u32 {
        self.end_episode();
        let id = self.observe(Kind::Episode, label, Some(emb), 0.0, Src::Measured);
        self.cur_ep = Some(id);
        self.last_ev = None;
        self.ep_events.clear();
        id
    }

    /// Registra un suceso dentro del episodio actual (abre uno si no existe).
    pub fn event(&mut self, label: &str, emb: &[f32; EMB], v: f32) -> u32 {
        if self.cur_ep.is_none() {
            let l = format!("ep{}", self.episodes.len());
            self.begin_episode(&l, emb);
        }
        let ep = self.cur_ep.unwrap();
        let ev = self.observe(Kind::Event, label, Some(emb), v, Src::Measured);
        self.link(ep, ev, Rel::Contains, 1.0);
        self.link(ev, ep, Rel::Contains, 0.5);
        if let Some((prev, tp)) = self.last_ev {
            if prev != ev && self.t.saturating_sub(tp) <= self.p.tau {
                let g = (-(self.t.saturating_sub(tp) as f32) / self.p.tau as f32).exp() + 0.5;
                self.link(prev, ev, Rel::Causal, g);
                self.link(prev, ev, Rel::Temporal, 1.0);
            }
        }
        self.last_ev = Some((ev, self.t));
        if !self.ep_events.contains(&ev) { self.ep_events.push(ev); }
        ev
    }

    pub fn end_episode(&mut self) {
        if self.cur_ep.take().is_some() && !self.ep_events.is_empty() {
            self.episodes.push(std::mem::take(&mut self.ep_events));
        }
        self.last_ev = None;
    }

    // ---- recuperación: activación propagada ----
    /// Semillas por similitud de huella neuronal + etiquetas exactas. Devuelve (id, activación) ordenado.
    pub fn recall(&mut self, seeds: &[(u32, f32)], emb: Option<&[f32; EMB]>) -> Vec<(u32, f32)> {
        let n = self.nodes.len();
        let mut a = vec![0f32; n];
        for &(i, x) in seeds { a[i as usize] = a[i as usize].max(x); }
        if let Some(e) = emb {
            for (i, nd) in self.nodes.iter().enumerate() {
                let s = cos(e, &nd.emb);
                if s > 0.8 { a[i] = a[i].max((s - 0.8) * 5.0 * 0.5); }
            }
        }
        for _ in 0..self.p.steps {
            let mut next: Vec<f32> = a.iter().map(|x| (1.0 - self.p.delta) * x).collect();
            for i in 0..n {
                if a[i] < 0.01 { continue; }
                let fan = self.adj[i].len().max(1) as f32;
                for e in &self.adj[i] {
                    let g = match e.rel { Rel::Causal | Rel::Predicts => 1.0, Rel::Grounds | Rel::Supports => 0.9, Rel::Contains => 0.6, _ => 0.5 };
                    next[e.to as usize] += a[i] * e.w * g / fan.sqrt();
                }
            }
            for i in 0..n {
                next[i] = (next[i] + 0.02 * (self.base(i as u32).max(-2.0) + 2.0) * (next[i] > 0.0) as i32 as f32).clamp(0.0, 1.0);
            }
            // inhibición lateral: k-WTA por tipo
            let mut by: HashMap<Kind, Vec<usize>> = HashMap::new();
            for i in 0..n { if next[i] > 0.0 { by.entry(self.nodes[i].kind).or_default().push(i); } }
            for (_, mut v) in by {
                if v.len() > self.p.k {
                    v.sort_by(|&x, &y| next[y].partial_cmp(&next[x]).unwrap());
                    for &i in &v[self.p.k..] { next[i] *= 0.3; }
                }
            }
            a = next;
        }
        for (i, x) in a.iter().enumerate() { self.nodes[i].a = *x; }
        let mut out: Vec<(u32, f32)> = a.iter().enumerate().filter(|(_, &x)| x > 0.05).map(|(i, &x)| (i as u32, x)).collect();
        out.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
        // recordar refuerza (y hace lábil): un acceso por cada nodo recuperado en el top
        for &(i, _) in out.iter().take(5) { let t = self.t; self.nodes[i as usize].accesses.push(t); }
        out
    }

    // ---- sueño ----
    pub fn sleep(&mut self) -> SleepReport {
        self.end_episode();
        let (mut concepts, mut skills, mut dreamed) = (0, 0, 0);
        // NREM: eventos presentes en >= min_support episodios → Concepto que los sostiene
        let mut support: HashMap<u32, Vec<usize>> = HashMap::new();
        for (ei, evs) in self.episodes.iter().enumerate() { for &e in evs { support.entry(e).or_default().push(ei); } }
        let mut ids: Vec<_> = support.iter().filter(|(_, v)| v.len() >= self.p.min_support).map(|(&k, _)| k).collect();
        ids.sort();
        for ev in ids {
            let label = format!("concepto:{}", self.nodes[ev as usize].label);
            let fresh = self.find(Kind::Concept, &label).is_none();
            let c = self.observe(Kind::Concept, &label, None, self.nodes[ev as usize].v, Src::Inferred);
            self.link(c, ev, Rel::Supports, 1.0);
            self.link(ev, c, Rel::Supports, 1.0);
            if fresh { concepts += 1; }
        }
        // habilidades: pares causales fuertes y repetidos → Skill(A→B)
        let snapshot: Vec<(u32, Edge)> = self.adj.iter().enumerate()
            .flat_map(|(i, l)| { let ms = self.p.min_support; let is_ev = self.nodes[i].kind == Kind::Event; l.iter().filter(move |e| is_ev && e.rel == Rel::Causal && e.n as usize >= ms).map(move |e| (i as u32, e.clone())) }).collect();
        for (a, e) in snapshot {
            let label = format!("habilidad:{}→{}", self.nodes[a as usize].label, self.nodes[e.to as usize].label);
            let fresh = self.find(Kind::Skill, &label).is_none();
            let s = self.observe(Kind::Skill, &label, None, 0.0, Src::Inferred);
            self.link(s, a, Rel::Involves, 1.0);
            self.link(s, e.to, Rel::Involves, 1.0);
            if fresh { skills += 1; }
        }
        // REM: A→B y B→C (causales) ⇒ A→C soñada, a comprobar despierto
        let causal: Vec<(u32, u32, f32)> = self.adj.iter().enumerate()
            .flat_map(|(i, l)| l.iter().filter(|e| e.rel == Rel::Causal && e.w > 0.25).map(move |e| (i as u32, e.to, e.w))).collect();
        for &(a, b, w1) in &causal {
            for &(b2, c, w2) in &causal {
                if b == b2 && a != c && !self.adj[a as usize].iter().any(|e| e.to == c && e.rel == Rel::Predicts) && dreamed < 32 {
                    self.adj[a as usize].push(Edge { to: c, w: 0.5 * w1.min(w2), rel: Rel::Predicts, last: self.t, n: 1 });
                    dreamed += 1;
                }
            }
        }
        // olvido: decaimiento y poda (nada se borra de golpe: primero se degrada)
        let mut pruned = 0;
        for l in self.adj.iter_mut() {
            for e in l.iter_mut() { e.w *= 1.0 - self.p.lambda; }
            let before = l.len();
            l.retain(|e| e.w >= self.p.prune);
            pruned += before - l.len();
        }
        SleepReport { concepts, skills, dreamed, pruned }
    }

    pub fn stats(&self) -> (usize, usize) { (self.nodes.len(), self.adj.iter().map(|l| l.len()).sum()) }

    /// Subgrafo activo para mostrar: (id, tipo, etiqueta, activación, fuente)
    pub fn top(&self, n: usize) -> Vec<(u32, Kind, String, f32, Src)> {
        let mut v: Vec<_> = self.nodes.iter().enumerate().filter(|(_, x)| x.a > 0.02).map(|(i, x)| (i as u32, x.kind, x.label.clone(), x.a, x.src)).collect();
        v.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
        v.truncate(n);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const E: [f32; EMB] = [0.0; EMB];

    fn episode(s: &mut Samn, evs: &[&str]) {
        s.begin_episode(&format!("ep{}", s.episodes.len()), &E);
        for e in evs { s.tick(); s.event(e, &E, 0.0); }
        s.end_episode();
    }

    #[test]
    fn repeticion_fortalece_y_falta_de_uso_olvida() {
        let mut s = Samn::new();
        for _ in 0..6 { episode(&mut s, &["abrir_editor", "escribir"]); }
        let a = s.find(Kind::Event, "abrir_editor").unwrap();
        let w = s.adj[a as usize].iter().find(|e| e.rel == Rel::Causal).unwrap().w;
        assert!(w > 0.5, "w={w}");
        for _ in 0..200 { s.sleep(); }
        assert!(s.adj[a as usize].iter().all(|e| e.rel != Rel::Causal), "debió olvidarse");
    }

    #[test]
    fn nrem_consolida_concepto_y_habilidad() {
        let mut s = Samn::new();
        for _ in 0..4 { episode(&mut s, &["guardar", "dialogo"]); }
        let r = s.sleep();
        assert!(r.concepts >= 2 && r.skills >= 1, "concepts={} skills={}", r.concepts, r.skills);
        assert!(s.find(Kind::Skill, "habilidad:guardar→dialogo").is_some());
    }

    #[test]
    fn rem_sueña_transitividad_marcada() {
        let mut s = Samn::new();
        for _ in 0..3 { episode(&mut s, &["A", "B"]); episode(&mut s, &["B", "C"]); }
        let r = s.sleep();
        let a = s.find(Kind::Event, "A").unwrap();
        let c = s.find(Kind::Event, "C").unwrap();
        assert!(r.dreamed >= 1);
        assert!(s.adj[a as usize].iter().any(|e| e.to == c && e.rel == Rel::Predicts));
    }

    #[test]
    fn recuperacion_propaga_y_recuerda_lo_asociado() {
        let mut s = Samn::new();
        for _ in 0..5 { episode(&mut s, &["hola", "saludo_usuario"]); }
        episode(&mut s, &["compilar", "error"]);
        let h = s.find(Kind::Event, "hola").unwrap();
        let out = s.recall(&[(h, 1.0)], None);
        let labels: Vec<_> = out.iter().map(|(i, _)| s.nodes[*i as usize].label.as_str()).collect();
        assert!(labels.contains(&"saludo_usuario"));
        assert!(!labels.contains(&"error"));
    }

    #[test]
    fn inhibicion_lateral_limita_por_tipo() {
        let mut s = Samn::new();
        let hub = s.observe(Kind::Concept, "hub", None, 0.0, Src::Measured);
        for i in 0..30 {
            let x = s.observe(Kind::Event, &format!("e{i}"), None, 0.0, Src::Measured);
            for _ in 0..5 { s.link(hub, x, Rel::Supports, 1.0); }
        }
        let out = s.recall(&[(hub, 1.0)], None);
        let strong = out.iter().filter(|(i, a)| s.nodes[*i as usize].kind == Kind::Event && *a > 0.2).count();
        assert!(strong <= s.p.k + 1, "strong={strong}");
    }

    #[test]
    fn persiste_y_recarga() {
        let mut s = Samn::new();
        for _ in 0..3 { episode(&mut s, &["x", "y"]); }
        let p = std::env::temp_dir().join("samn_test.json");
        s.save(&p).unwrap();
        let r = Samn::load(&p).unwrap();
        assert_eq!(r.stats(), s.stats());
        assert!(r.find(Kind::Event, "x").is_some());
    }
}
