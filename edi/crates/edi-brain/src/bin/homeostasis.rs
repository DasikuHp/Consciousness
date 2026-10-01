//! Plasticidad homeostática intrínseca de las células de Kenyon (como en el cerebro real):
//! cada KC ajusta su umbral para disparar en ~5 % de los olores. Se calibra con olores aleatorios
//! (3 glomérulos reales de 53) y se mide la especificidad (Jaccard entre olores) antes y después.
//! Guarda los desplazamientos en <dir>/kc_thr_offset.f32 (estado aprendido, no anatomía).
//! Uso: homeostasis <dir_conectoma> [rondas]

use edi_brain::{Brain, Connectome, Params};
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

const P_TARGET: f32 = 0.05;
const ETA: f32 = 1.0;

fn brain<'c>(c: &'c Connectome, off: &[f32]) -> Brain<'c> {
    let mut b = Brain::new(c, Params { dt: 0.25, ..Params::default() });
    b.std_dep = Some((0.3, 100.0));
    b.sfa = Some((2.0, 200.0));
    b.thr_offset = Some(off.to_vec());
    b
}

fn kc_set(b: &mut Brain, odor: &[(u32, f32)], kc: &[u32], rng: &mut ChaCha8Rng) -> HashSet<u32> {
    b.reset_counts();
    b.run(300.0, odor, rng);
    let s = kc.iter().cloned().filter(|&k| b.counts[k as usize] > 0).collect();
    b.run(1000.0, &[], rng);
    s
}

fn jaccard(a: &HashSet<u32>, b: &HashSet<u32>) -> f64 {
    let u = a.union(b).count();
    if u == 0 { f64::NAN } else { a.intersection(b).count() as f64 / u as f64 }
}

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&a[1]);
    let rounds: usize = a.get(2).and_then(|x| x.parse().ok()).unwrap_or(80);
    let c = Connectome::load(dir)?;
    let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("meta.json"))?)?;
    let types: Vec<String> = meta["orn_types"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).filter(|t| !c.group(t).is_empty()).collect();
    let kc = c.group("kenyon_cell").to_vec();
    let odor = |ts: &[&String]| ts.iter().flat_map(|t| c.group(t).to_vec()).map(|i| (i, 60.0f32)).collect::<Vec<_>>();
    let mut rng = ChaCha8Rng::seed_from_u64(9);
    // pares de olores de prueba fijos (no usados en calibración)
    let pairs: Vec<(Vec<(u32, f32)>, Vec<(u32, f32)>)> = (0..6).map(|_| {
        let mut t = types.clone(); t.shuffle(&mut rng);
        (odor(&[&t[0], &t[1], &t[2]]), odor(&[&t[3], &t[4], &t[5]]))
    }).collect();
    let evaluate = |off: &[f32], rng: &mut ChaCha8Rng| {
        let (mut jac, mut frac) = (0.0, 0.0);
        for (oa, ob) in &pairs {
            let sa = kc_set(&mut brain(&c, off), oa, &kc, rng);
            let sb = kc_set(&mut brain(&c, off), ob, &kc, rng);
            jac += jaccard(&sa, &sb);
            frac += (sa.len() + sb.len()) as f64 / (2.0 * kc.len() as f64);
        }
        (jac / pairs.len() as f64, frac / pairs.len() as f64)
    };
    let mut off = vec![0f32; c.n];
    let (j0, f0) = evaluate(&off, &mut rng);
    println!("antes: KC activas por olor {:.1} % · solapamiento (Jaccard) entre olores distintos {:.3}", 100.0 * f0, j0);
    let mut b = brain(&c, &off);
    for r in 0..rounds {
        let mut t = types.clone(); t.shuffle(&mut rng);
        let s = kc_set(&mut b, &odor(&[&t[0], &t[1], &t[2]]), &kc, &mut rng);
        for &k in &kc {
            let fired = if s.contains(&k) { 1.0 } else { 0.0 };
            off[k as usize] = (off[k as usize] + ETA * (fired - P_TARGET)).clamp(-8.0, 40.0);
        }
        b.thr_offset = Some(off.clone());
        if (r + 1) % 20 == 0 {
            let (j, f) = evaluate(&off, &mut rng);
            println!("ronda {:>3}: KC activas {:.1} % · Jaccard {:.3}", r + 1, 100.0 * f, j);
        }
    }
    let raw: Vec<u8> = off.iter().flat_map(|x| x.to_le_bytes()).collect();
    std::fs::write(dir.join("kc_thr_offset.f32"), raw)?;
    Ok(())
}
