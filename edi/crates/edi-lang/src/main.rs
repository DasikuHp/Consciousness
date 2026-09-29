//! Connectome-RWKV v0: la mosca ES la red recurrente.
//!
//! Plano RWKV-7: token-shift en la entrada, estado recurrente de tamaño fijo que
//! nunca se reinicia dentro de la secuencia y lectura lineal (canal de salida).
//! El estado y la recurrencia son el conectoma real (LIF): los caracteres entran
//! como tasas Poisson al órgano de Johnston (oído) y se leen de las neuronas
//! descendentes. Desde cero: solo evolucionan E (entrada), W (salida), b y mu,
//! con EGGROLL (ES con perturbaciones de rango bajo). La anatomía no se toca.
//!
//! Uso: edi-lang <dir_conectoma> <out_dir> <modo: real|shuffle|bypass> <generaciones> [seed]

use edi_brain::{Brain, Connectome, Params};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::StandardNormal;
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;

const CORPUS: &str = "hola. soy edi. hola edi. te oigo. te veo. hola, ¿cómo estás? estoy bien. \
soy una mosca que aprende a hablar. aprendo contigo. hablo poco. hola, soy edi. \
¿quién eres? eres mi amigo. yo soy edi. tú eres tú. me gusta aprender. \
oigo tu voz. veo la pantalla. sí. no. hola. adiós. soy edi. ";

const T_TOK_MS: f64 = 10.0;
const R_MAX: f32 = 250.0;
const SEQ: usize = 32;
const POP: usize = 24;
const RANK: usize = 2;
const SIGMA: f32 = 0.1;
const LR: f32 = 0.01;
const DECAY: f32 = 1e-3;

#[derive(Clone)]
struct Theta {
    e: Vec<f32>,   // V x nin
    w: Vec<f32>,   // V x nf
    b: Vec<f32>,   // V
    mu: f32,
}

struct Task {
    vocab: Vec<char>,
    ids: Vec<usize>,
    jo: Vec<u32>,
    ro: Vec<u32>,
}

impl Task {
    /// Residual (entrada directa) + estado del cerebro (conteos + traza) de las neuronas de lectura.
    fn nf(&self, bypass: bool) -> usize {
        if bypass { self.jo.len() } else { self.jo.len() + 2 * self.ro.len() }
    }
}

fn softmax_ce(logits: &[f32], y: usize) -> (f32, usize) {
    let m = logits.iter().cloned().fold(f32::MIN, f32::max);
    let z: f32 = logits.iter().map(|l| (l - m).exp()).sum();
    let arg = logits.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
    (-(logits[y] - m - z.ln()), arg)
}

/// Ejecuta el cerebro sobre `seq` y devuelve (CE media, aciertos, predicciones).
fn run(c: &Connectome, t: &Task, th: &Theta, seq: &[usize], seed: u64, bypass: bool) -> (f32, usize, Vec<usize>) {
    let (v, nin, nf) = (t.vocab.len(), t.jo.len(), t.nf(bypass));
    let p = Params { dt: 0.5, ..Params::default() };
    let mut brain = Brain::new(c, p);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut trace = vec![0f32; t.ro.len()];
    let (mut ce, mut hit, mut preds) = (0f32, 0usize, Vec::new());
    let mut prev = seq[0];
    for k in 0..seq.len() - 1 {
        let x = seq[k];
        let rates: Vec<f32> = (0..nin).map(|j| {
            let a = th.e[x * nin + j] + th.mu * th.e[prev * nin + j] - 2.0;
            R_MAX / (1.0 + (-a).exp())
        }).collect();
        prev = x;
        let mut feat: Vec<f32> = rates.iter().map(|r| r / R_MAX).collect();
        if !bypass {
            let drive: Vec<(u32, f32)> = t.jo.iter().zip(&rates).map(|(&i, &r)| (i, r)).collect();
            brain.reset_counts();
            brain.run(T_TOK_MS, &drive, &mut rng);
            let mut tr_part = Vec::with_capacity(t.ro.len());
            for (d, tr) in t.ro.iter().zip(trace.iter_mut()) {
                let cnt = brain.counts[*d as usize] as f32;
                *tr = 0.7 * *tr + cnt;
                feat.push(cnt / 4.0);
                tr_part.push(*tr / 12.0);
            }
            feat.extend(tr_part);
        }
        let logits: Vec<f32> = (0..v).map(|o| {
            th.b[o] + th.w[o * nf..(o + 1) * nf].iter().zip(&feat).map(|(a, b)| a * b).sum::<f32>()
        }).collect();
        let (l, arg) = softmax_ce(&logits, seq[k + 1]);
        if k >= 2 {
            ce += l;
            hit += (arg == seq[k + 1]) as usize;
        }
        preds.push(arg);
    }
    (ce / (seq.len() - 3) as f32, hit, preds)
}

/// Perturbación de rango bajo A·Bᵀ/√r para una matriz rows×cols.
fn lowrank(rows: usize, cols: usize, rng: &mut ChaCha8Rng) -> Vec<f32> {
    let a: Vec<f32> = (0..rows * RANK).map(|_| rng.sample(StandardNormal)).collect();
    let bm: Vec<f32> = (0..cols * RANK).map(|_| rng.sample(StandardNormal)).collect();
    let s = 1.0 / (RANK as f32).sqrt();
    let mut m = vec![0f32; rows * cols];
    for i in 0..rows {
        for j in 0..cols {
            m[i * cols + j] = s * (0..RANK).map(|r| a[i * RANK + r] * bm[j * RANK + r]).sum::<f32>();
        }
    }
    m
}

/// Sonda: estimula el oído (JO) 300 ms y elige las N_RO neuronas no-JO más activas:
/// la vía auditiva real que alcanza la señal en ese conectoma.
const N_RO: usize = 1024;
fn probe(c: &Connectome, jo: &[u32]) -> Vec<u32> {
    let mut b = Brain::new(c, Params { dt: 0.5, ..Params::default() });
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    let drive: Vec<(u32, f32)> = jo.iter().map(|&i| (i, 120.0)).collect();
    b.run(300.0, &drive, &mut rng);
    let jset: std::collections::HashSet<u32> = jo.iter().cloned().collect();
    let mut act: Vec<(u32, u32)> = b.counts.iter().enumerate()
        .filter(|(i, &k)| k > 0 && !jset.contains(&(*i as u32))).map(|(i, &k)| (i as u32, k)).collect();
    act.sort_by(|a, b| b.1.cmp(&a.1));
    act.truncate(N_RO);
    act.into_iter().map(|x| x.0).collect()
}

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let dir = PathBuf::from(&a[1]);
    let out = PathBuf::from(&a[2]);
    let mode = a[3].as_str();
    let gens: usize = a[4].parse()?;
    let seed: u64 = a.get(5).map(|s| s.parse().unwrap()).unwrap_or(1);
    std::fs::create_dir_all(&out)?;
    let mut c = Connectome::load(&dir)?;
    if mode == "shuffle" {
        c = c.shuffled(seed + 100);
    }
    let bypass = mode == "bypass";
    let mut vocab: Vec<char> = CORPUS.chars().collect();
    vocab.sort();
    vocab.dedup();
    let ids: Vec<usize> = CORPUS.chars().map(|ch| vocab.binary_search(&ch).unwrap()).collect();
    let jo = c.group("auditory_jo").to_vec();
    let ro = probe(&c, &jo);
    let task = Task { vocab, ids, jo, ro };
    let (v, nin, nf) = (task.vocab.len(), task.jo.len(), task.nf(bypass));
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut th = Theta {
        e: (0..v * nin).map(|_| rng.sample::<f32, _>(StandardNormal) * 0.5).collect(),
        w: vec![0.0; v * nf],
        b: vec![0.0; v],
        mu: 0.5,
    };
    // Baselines de la tarea: unigrama y bigrama (entropía cruzada en el corpus).
    let n = task.ids.len();
    let mut uni = vec![1f64; v];
    let mut bi = vec![vec![1f64; v]; v];
    for k in 0..n - 1 {
        uni[task.ids[k + 1]] += 1.0;
        bi[task.ids[k]][task.ids[k + 1]] += 1.0;
    }
    let su: f64 = uni.iter().sum();
    let ce_uni = (0..n - 1).map(|k| -(uni[task.ids[k + 1]] / su).ln()).sum::<f64>() / (n - 1) as f64;
    let ce_bi = (0..n - 1).map(|k| {
        let row = &bi[task.ids[k]];
        -(row[task.ids[k + 1]] / row.iter().sum::<f64>()).ln()
    }).sum::<f64>() / (n - 1) as f64;
    let mut log = std::fs::File::create(out.join("log.jsonl"))?;
    writeln!(log, "{}", serde_json::json!({"mode": mode, "vocab": v, "nin": nin, "nf": nf, "n_readout_reached": task.ro.len(), "ce_unigram": ce_uni, "ce_bigram": ce_bi, "ce_uniform": (v as f64).ln()}))?;
    let eval_offsets = [0usize, 60, 130, 200];
    let (mut mom_e, mut mom_w, mut mom_b) = (vec![0f32; v * nin], vec![0f32; v * nf], vec![0f32; v]);
    for g in 0..gens {
        let off = rng.gen_range(0..n - SEQ);
        let seq = &task.ids[off..off + SEQ];
        let noise_seed: u64 = rng.gen();
        let eps: Vec<(Vec<f32>, Vec<f32>, Vec<f32>)> = (0..POP / 2).map(|_| {
            (lowrank(v, nin, &mut rng), lowrank(v, nf, &mut rng), (0..v).map(|_| rng.sample(StandardNormal)).collect())
        }).collect();
        let fit: Vec<f32> = (0..POP).into_par_iter().map(|i| {
            let (de, dw, db) = &eps[i / 2];
            let sgn = if i % 2 == 0 { SIGMA } else { -SIGMA };
            let mut t2 = th.clone();
            t2.e.iter_mut().zip(de).for_each(|(x, d)| *x += sgn * d);
            t2.w.iter_mut().zip(dw).for_each(|(x, d)| *x += sgn * d);
            t2.b.iter_mut().zip(db).for_each(|(x, d)| *x += sgn * d);
            -run(&c, &task, &t2, seq, noise_seed, bypass).0
        }).collect();
        // rangos centrados (robusto al escalado de la fitness)
        let mut order: Vec<usize> = (0..POP).collect();
        order.sort_by(|&x, &y| fit[x].partial_cmp(&fit[y]).unwrap());
        let mut shaped = vec![0f32; POP];
        for (r, &i) in order.iter().enumerate() {
            shaped[i] = r as f32 / (POP - 1) as f32 - 0.5;
        }
        let scale = LR / (POP as f32 * SIGMA);
        let upd = |m: &mut Vec<f32>, th_v: &mut Vec<f32>, which: usize| {
            for (k, x) in m.iter_mut().enumerate() {
                let gsum: f32 = (0..POP / 2).map(|p| {
                    let e = match which { 0 => &eps[p].0, 1 => &eps[p].1, _ => &eps[p].2 };
                    (shaped[2 * p] - shaped[2 * p + 1]) * e[k]
                }).sum();
                *x = 0.5 * *x + scale * gsum;
            }
            th_v.iter_mut().zip(m.iter()).for_each(|(t, d)| *t = *t * (1.0 - DECAY) + d);
        };
        upd(&mut mom_e, &mut th.e, 0);
        upd(&mut mom_w, &mut th.w, 1);
        upd(&mut mom_b, &mut th.b, 2);
        if g % 10 == 0 || g + 1 == gens {
            let evals: Vec<(f32, usize, Vec<usize>)> = eval_offsets.par_iter()
                .map(|&o| run(&c, &task, &th, &task.ids[o..o + SEQ], 999, bypass)).collect();
            let ce = evals.iter().map(|e| e.0).sum::<f32>() / evals.len() as f32;
            let acc = evals.iter().map(|e| e.1).sum::<usize>() as f32 / (evals.len() * (SEQ - 3)) as f32;
            let said: String = evals[0].2.iter().map(|&i| task.vocab[i]).collect();
            let heard: String = task.ids[1..SEQ].iter().map(|&i| task.vocab[i]).collect();
            writeln!(log, "{}", serde_json::json!({"gen": g, "train_fit": fit.iter().sum::<f32>() / POP as f32, "eval_ce": ce, "eval_acc": acc, "heard": heard, "said": said}))?;
            log.flush()?;
            println!("[{mode}] gen {g:4} ce={ce:.3} acc={acc:.3} (uni={ce_uni:.3} bi={ce_bi:.3}) dice: {said:?}");
        }
    }
    let raw: Vec<u8> = th.e.iter().chain(&th.w).chain(&th.b).flat_map(|x| x.to_le_bytes()).collect();
    std::fs::write(out.join("theta.f32"), raw)?;
    Ok(())
}
