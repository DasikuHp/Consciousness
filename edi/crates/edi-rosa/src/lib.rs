//! ROSA (Rapid Online Suffix Automaton, RWKV-8 "Heron", BlinkDL) en Rust.
//!
//! Port fiel de `rosa()` de `external/lang/RWKV-LM/RWKV-v8/251014_rosa_1bit_layer.py`:
//! tras ver x[0..=i], predice x[i+1] como el token que siguió a la aparición previa más
//! reciente del sufijo más largo de la historia que ya había aparecido. Es memoria
//! episódica exacta e infinita (sin pesos), O(1) amortizado por token.
//!
//! En EDI: (1) memoria de secuencias de lo vivido (palabras, ventanas, puertos…) que
//! predice lo siguiente y recuerda qué vino después; (2) ROSA de 1 bit por canal sobre la
//! actividad del cerebro (como `ROSA_1bit`), cuyo error de predicción es su sorpresa.

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Rosa {
    /// Historia completa (la memoria; el autómata es solo su índice).
    pub hist: Vec<u32>,
    /// Ventana máxima (0 = sin límite): al superar 2·window se reconstruye con la última mitad.
    pub window: usize,
    #[serde(skip)] next: Vec<Vec<(u32, u32)>>,
    #[serde(skip)] link: Vec<i32>,
    #[serde(skip)] len: Vec<u32>,
    #[serde(skip)] end: Vec<i64>,
    #[serde(skip)] last: u32,
    #[serde(skip)] built: usize,
}

fn get(m: &[(u32, u32)], t: u32) -> Option<u32> { m.iter().find(|e| e.0 == t).map(|e| e.1) }
fn set(m: &mut Vec<(u32, u32)>, t: u32, s: u32) {
    match m.iter_mut().find(|e| e.0 == t) { Some(e) => e.1 = s, None => m.push((t, s)) }
}

/// Predicción: siguiente token esperado y longitud del contexto que coincide (confianza).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pred { pub token: u32, pub ctx: u32, pub at: usize }

impl Rosa {
    pub fn new(window: usize) -> Self { let mut r = Rosa { window, ..Default::default() }; r.reset(); r }

    fn reset(&mut self) {
        self.next = vec![vec![]]; self.link = vec![-1]; self.len = vec![0]; self.end = vec![-1]; self.last = 0; self.built = 0;
    }

    /// Reconstruye el índice desde `hist` (tras cargar de disco).
    pub fn rebuild(&mut self) {
        let h = std::mem::take(&mut self.hist);
        self.reset();
        for t in h { self.push(t); }
    }

    /// Añade un token y devuelve la predicción del siguiente (None si no hay contexto previo).
    pub fn push(&mut self, t: u32) -> Option<Pred> {
        if self.next.is_empty() { self.reset(); } // creada con Default
        if self.window > 0 && self.hist.len() >= 2 * self.window {
            let keep = self.hist.split_off(self.hist.len() - self.window);
            self.hist = keep;
            self.rebuild();
        }
        let i = self.hist.len() as i64;
        self.hist.push(t);
        let r = self.next.len() as u32;
        self.next.push(vec![]); self.link.push(-1); self.len.push(self.len[self.last as usize] + 1); self.end.push(-1);
        let mut p = self.last as i32;
        while p != -1 && get(&self.next[p as usize], t).is_none() { set(&mut self.next[p as usize], t, r); p = self.link[p as usize]; }
        if p == -1 { self.link[r as usize] = 0; } else {
            let q = get(&self.next[p as usize], t).unwrap();
            if self.len[p as usize] + 1 == self.len[q as usize] { self.link[r as usize] = q as i32; } else {
                let u = self.next.len() as u32;
                self.next.push(self.next[q as usize].clone()); self.len.push(self.len[p as usize] + 1);
                self.link.push(self.link[q as usize]); self.end.push(self.end[q as usize]);
                while p != -1 && get(&self.next[p as usize], t) == Some(q) { set(&mut self.next[p as usize], t, u); p = self.link[p as usize]; }
                self.link[q as usize] = u as i32; self.link[r as usize] = u as i32;
            }
        }
        self.last = r;
        let mut v = r as i32;
        let mut out = None;
        while v != -1 {
            let e = self.end[v as usize];
            if self.len[v as usize] > 0 && e >= 0 { out = Some(Pred { token: self.hist[e as usize + 1], ctx: self.len[v as usize], at: e as usize + 1 }); break; }
            v = self.link[v as usize];
        }
        let mut v = r as i32;
        while v != -1 && self.end[v as usize] < i { self.end[v as usize] = i; v = self.link[v as usize]; }
        self.built = self.hist.len();
        out
    }

    /// Recuerdo episódico exacto: lo que vino después la última vez que ocurrió `pred` (k tokens).
    pub fn recall(&self, pred: &Pred, k: usize) -> &[u32] { &self.hist[pred.at..(pred.at + k).min(self.hist.len())] }

    /// Dónde apareció por última vez el token `t` (para recordar «qué vino después de X»).
    pub fn after(&self, t: u32, k: usize) -> &[u32] {
        match self.hist.iter().rposition(|&x| x == t) {
            Some(j) if j + 1 < self.hist.len() => &self.hist[j + 1..(j + 1 + k).min(self.hist.len())],
            _ => &[],
        }
    }

    pub fn from_hist(hist: Vec<u32>, window: usize) -> Self { let mut r = Rosa { hist, window, ..Default::default() }; r.rebuild(); r }

    pub fn states(&self) -> usize { self.next.len() }
}

/// Vocabulario: etiquetas ↔ tokens (para que la historia sea legible y persistente).
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Vocab { pub labels: Vec<String>, #[serde(skip)] idx: std::collections::HashMap<String, u32> }

impl Vocab {
    pub fn id(&mut self, s: &str) -> u32 {
        if self.idx.len() != self.labels.len() { self.idx = self.labels.iter().enumerate().map(|(i, l)| (l.clone(), i as u32)).collect(); }
        if let Some(&i) = self.idx.get(s) { return i; }
        self.labels.push(s.to_string());
        let i = self.labels.len() as u32 - 1;
        self.idx.insert(s.to_string(), i);
        i
    }
    pub fn get(&self, s: &str) -> Option<u32> { self.labels.iter().position(|l| l == s).map(|i| i as u32) }
    pub fn label(&self, i: u32) -> &str { &self.labels[i as usize] }
}

/// ROSA de 1 bit por canal (como `ROSA_1bit` de RWKV-8): cada canal es una secuencia de bits.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Rosa1Bit { pub ch: Vec<Rosa>, #[serde(skip)] pub pred: Vec<Option<u32>> }

impl Rosa1Bit {
    pub fn new(c: usize, window: usize) -> Self { Rosa1Bit { ch: (0..c).map(|_| Rosa::new(window)).collect(), pred: vec![None; c] } }
    /// Empuja un vector de bits; devuelve (aciertos, predichos) respecto a la predicción anterior.
    pub fn push(&mut self, bits: &[bool]) -> (usize, usize) {
        if self.pred.len() != self.ch.len() { self.pred = vec![None; self.ch.len()]; }
        let (mut hit, mut tot) = (0, 0);
        for (k, &b) in bits.iter().enumerate() {
            if let Some(p) = self.pred[k] { tot += 1; if p == b as u32 { hit += 1; } }
            self.pred[k] = self.ch[k].push(b as u32).map(|p| p.token);
        }
        (hit, tot)
    }
}

impl Rosa {
    /// Todas las continuaciones del contexto más largo (hasta `max_ctx` tokens) que ya apareció:
    /// (token, veces), de más a menos frecuente. Es lo que "la gente dijo después de esto".
    pub fn continuations(&self, ctx: &[u32], max_ctx: usize) -> Vec<(u32, u32)> {
        let h = &self.hist;
        for l in (1..=max_ctx.min(ctx.len())).rev() {
            let pat = &ctx[ctx.len() - l..];
            let mut out: Vec<(u32, u32)> = vec![];
            for j in l..h.len() {
                if &h[j - l..j] == pat {
                    match out.iter_mut().find(|e| e.0 == h[j]) { Some(e) => e.1 += 1, None => out.push((h[j], 1)) }
                }
            }
            if !out.is_empty() { out.sort_by(|a, b| b.1.cmp(&a.1)); return out; }
        }
        vec![]
    }
}
