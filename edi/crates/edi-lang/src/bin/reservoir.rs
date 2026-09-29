//! Prueba de reservorio: ¿el estado del cerebro real contiene información para predecir
//! la siguiente letra que NO está en la entrada inmediata?
//!
//! Misma codificación de entrada (fija, aleatoria) para todas las condiciones. Se recorre
//! el corpus una vez con el cerebro corriendo sin reiniciar (estado = memoria), se extraen
//! rasgos por carácter y se ajusta una lectura lineal óptima (ridge, λ por validación).
//! Condiciones: bypass (solo entrada actual+previa), real (+cerebro), barajado (+cerebro
//! con conectoma barajado), solo-cerebro. Evaluado en el último 25 % del texto (no visto).
//! Uso: reservoir <dir_conectoma> <out.json>

use edi_brain::{Brain, Connectome, Params};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::StandardNormal;
use std::collections::HashSet;

const CORPUS: &str = include_str!("../corpus_es.txt");
const T_TOK_MS: f64 = 10.0;
const R_MAX: f32 = 250.0;

fn probe(c: &Connectome, jo: &[u32], n: usize) -> Vec<u32> {
    let mut b = Brain::new(c, Params { dt: 0.5, ..Params::default() });
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    let drive: Vec<(u32, f32)> = jo.iter().map(|&i| (i, 120.0)).collect();
    b.run(300.0, &drive, &mut rng);
    let js: HashSet<u32> = jo.iter().cloned().collect();
    let mut a: Vec<(u32, u32)> = b.counts.iter().enumerate().filter(|(i, &k)| k > 0 && !js.contains(&(*i as u32))).map(|(i, &k)| (i as u32, k)).collect();
    a.sort_by(|x, y| y.1.cmp(&x.1));
    a.truncate(n);
    a.into_iter().map(|x| x.0).collect()
}

/// Rasgos por carácter: (residual [rates actuales|previos], cerebro [conteo, traza 0.6, 0.85, 0.95]).
fn features(c: &Connectome, ids: &[usize], e: &[f32], v: usize, jo: &[u32], ro: &[u32], seed: u64) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    let nin = jo.len();
    let mut brain = Brain::new(c, Params { dt: 0.5, ..Params::default() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let taus = [0.6f32, 0.85, 0.95];
    let mut tr = vec![vec![0f32; ro.len()]; taus.len()];
    let (mut res, mut brn) = (Vec::new(), Vec::new());
    let rate = |x: usize| -> Vec<f32> { (0..nin).map(|j| R_MAX / (1.0 + (-(e[x * nin + j] - 1.5)).exp())).collect() };
    let _ = v;
    let mut prev = ids[0];
    for &x in ids {
        let r = rate(x);
        let rp = rate(prev);
        prev = x;
        let drive: Vec<(u32, f32)> = jo.iter().zip(&r).map(|(&i, &q)| (i, q)).collect();
        brain.reset_counts();
        brain.run(T_TOK_MS, &drive, &mut rng);
        let mut f = Vec::with_capacity(ro.len() * 4);
        for (k, &i) in ro.iter().enumerate() {
            let cnt = brain.counts[i as usize] as f32;
            f.push(cnt);
            for (t, &a) in taus.iter().enumerate() { tr[t][k] = a * tr[t][k] + (1.0 - a) * cnt; }
        }
        for t in &tr { f.extend(t.iter().cloned()); }
        res.push(r.iter().chain(rp.iter()).map(|q| q / R_MAX).collect());
        brn.push(f);
    }
    (res, brn)
}

/// Ridge multiclase: W = (XᵀX + λI)⁻¹ XᵀY, con Cholesky.
fn ridge(x: &[Vec<f32>], y: &[usize], v: usize, lambda: f64) -> Vec<Vec<f64>> {
    let d = x[0].len();
    let mut a = vec![0f64; d * d];
    let mut b = vec![0f64; d * v];
    for (row, &t) in x.iter().zip(y) {
        for i in 0..d {
            let xi = row[i] as f64;
            if xi == 0.0 { continue; }
            for j in 0..=i { a[i * d + j] += xi * row[j] as f64; }
            b[i * v + t] += xi;
        }
    }
    for i in 0..d { for j in 0..i { a[j * d + i] = a[i * d + j]; } a[i * d + i] += lambda; }
    // Cholesky in-place (inferior)
    for j in 0..d {
        let mut s = a[j * d + j];
        for k in 0..j { s -= a[j * d + k] * a[j * d + k]; }
        let l = s.max(1e-12).sqrt();
        a[j * d + j] = l;
        for i in j + 1..d {
            let mut s = a[i * d + j];
            for k in 0..j { s -= a[i * d + k] * a[j * d + k]; }
            a[i * d + j] = s / l;
        }
    }
    let mut w = vec![vec![0f64; d]; v];
    for o in 0..v {
        let mut z = vec![0f64; d];
        for i in 0..d { let mut s = b[i * v + o]; for k in 0..i { s -= a[i * d + k] * z[k]; } z[i] = s / a[i * d + i]; }
        for i in (0..d).rev() { let mut s = z[i]; for k in i + 1..d { s -= a[k * d + i] * w[o][k]; } w[o][i] = s / a[i * d + i]; }
    }
    w
}

/// Entropía cruzada (softmax con temperatura ajustada en validación) y acierto.
fn eval(w: &[Vec<f64>], x: &[Vec<f32>], y: &[usize], temp: f64) -> (f64, f64) {
    let (mut ce, mut hit) = (0.0, 0usize);
    for (row, &t) in x.iter().zip(y) {
        let s: Vec<f64> = w.iter().map(|wo| wo.iter().zip(row).map(|(a, b)| a * *b as f64).sum::<f64>() * temp).collect();
        let m = s.iter().cloned().fold(f64::MIN, f64::max);
        let z: f64 = s.iter().map(|q| (q - m).exp()).sum();
        ce += -(s[t] - m - z.ln());
        let arg = s.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        hit += (arg == t) as usize;
    }
    (ce / y.len() as f64, hit as f64 / y.len() as f64)
}

fn fit(xtr: &[Vec<f32>], ytr: &[usize], xva: &[Vec<f32>], yva: &[usize], xte: &[Vec<f32>], yte: &[usize], v: usize) -> serde_json::Value {
    let mut best = (f64::MAX, 0.0, 0.0);
    for &l in &[0.1, 1.0, 10.0, 100.0, 1000.0] {
        let w = ridge(xtr, ytr, v, l);
        for &t in &[2.0, 5.0, 10.0, 20.0, 40.0] {
            let (ce, _) = eval(&w, xva, yva, t);
            if ce < best.0 { best = (ce, l, t); }
        }
    }
    let mut xall = xtr.to_vec(); xall.extend_from_slice(xva);
    let mut yall = ytr.to_vec(); yall.extend_from_slice(yva);
    let w = ridge(&xall, &yall, v, best.1);
    let (ce, acc) = eval(&w, xte, yte, best.2);
    serde_json::json!({"test_ce": ce, "test_acc": acc, "lambda": best.1, "temp": best.2, "dims": xtr[0].len()})
}

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let c = Connectome::load(std::path::Path::new(&a[1]))?;
    let text: String = CORPUS.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    let mut vocab: Vec<char> = text.chars().collect();
    vocab.sort();
    vocab.dedup();
    let ids: Vec<usize> = text.chars().map(|ch| vocab.binary_search(&ch).unwrap()).collect();
    let v = vocab.len();
    let jo = c.group("auditory_jo").to_vec();
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    let e: Vec<f32> = (0..v * jo.len()).map(|_| rng.sample::<f32, _>(StandardNormal) * 1.5).collect();
    let n = ids.len() - 1;
    let (i_va, i_te) = (n * 60 / 100, n * 75 / 100);
    let sh = c.shuffled(101);
    let mut out = serde_json::json!({"chars": n, "vocab": v});
    // n-gramas (entrenados en train+val) como referencia
    for k in 1..=3usize {
        let mut cnt: std::collections::HashMap<Vec<usize>, Vec<f64>> = Default::default();
        for i in k - 1..i_te { cnt.entry(ids[i + 1 - k..=i].to_vec()).or_insert(vec![0.1; v])[ids[i + 1]] += 1.0; }
        let (mut ce, mut hit) = (0.0, 0);
        for i in i_te..n {
            let ctx = ids[i + 1 - k..=i].to_vec();
            let row = cnt.get(&ctx).cloned().unwrap_or(vec![1.0; v]);
            let s: f64 = row.iter().sum();
            ce += -(row[ids[i + 1]] / s).ln();
            hit += (row.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0 == ids[i + 1]) as usize;
        }
        out[format!("ngram_{k}")] = serde_json::json!({"test_ce": ce / (n - i_te) as f64, "test_acc": hit as f64 / (n - i_te) as f64});
    }
    for (name, conn) in [("real", &c), ("barajado", &sh)] {
        let ro = probe(conn, &jo, 1024);
        let (res, brn) = features(conn, &ids[..n], &e, v, &jo, &ro, 11);
        let y: Vec<usize> = ids[1..=n].to_vec();
        let split = |xs: &Vec<Vec<f32>>| (xs[..i_va].to_vec(), xs[i_va..i_te].to_vec(), xs[i_te..].to_vec());
        let comb: Vec<Vec<f32>> = res.iter().zip(&brn).map(|(a, b)| a.iter().chain(b.iter()).cloned().collect()).collect();
        if name == "real" {
            let (a1, a2, a3) = split(&res);
            out["bypass"] = fit(&a1, &y[..i_va], &a2, &y[i_va..i_te], &a3, &y[i_te..], v);
            let (b1, b2, b3) = split(&brn);
            out["solo_cerebro_real"] = fit(&b1, &y[..i_va], &b2, &y[i_va..i_te], &b3, &y[i_te..], v);
        }
        let (c1, c2, c3) = split(&comb);
        out[format!("bypass+cerebro_{name}")] = fit(&c1, &y[..i_va], &c2, &y[i_va..i_te], &c3, &y[i_te..], v);
        out[format!("neuronas_lectura_{name}")] = serde_json::json!(ro.len());
        eprintln!("{name} listo");
    }
    std::fs::write(&a[2], serde_json::to_string_pretty(&out)?)?;
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
