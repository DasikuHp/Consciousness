//! Validación frente a Brian2 (Shiu et al.): azúcar (20 GRN, 150 Hz) → tasas por neurona.
//! Uso: validate <dir_conectoma> <n_trials> <out.json> [p_release] [shuffle]

use edi_brain::{entropy::{Entropy, Source}, Brain, Connectome, Params};
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let dir = PathBuf::from(&a[1]);
    let trials: usize = a[2].parse()?;
    let p_rel: Option<f64> = a.get(4).and_then(|s| s.parse().ok()).filter(|p: &f64| *p < 1.0);
    let mut c = Connectome::load(&dir)?;
    if a.get(5).map(|s| s == "shuffle").unwrap_or(false) {
        c = c.shuffled(7);
    }
    let drive: Vec<(u32, f32)> = c.group("sugar_shiu").iter().map(|&i| (i, 150.0)).collect();
    let mn9 = c.group("mn9")[0] as usize;
    let p = Params { p_release: p_rel, ..Params::default() };
    let t0 = Instant::now();
    let per_trial: Vec<Vec<u32>> = (0..trials).into_par_iter().map(|t| {
        let mut rng = Entropy::new(Source::Fixed(1000 + t as u64), None).unwrap().rng().unwrap();
        let mut b = Brain::new(&c, p.clone());
        b.run(1000.0, &drive, &mut rng);
        b.counts.clone()
    }).collect();
    let wall = t0.elapsed().as_secs_f64();
    let mut rate = vec![0f64; c.n];
    for cnt in &per_trial {
        for (r, &k) in rate.iter_mut().zip(cnt) {
            *r += k as f64 / trials as f64;
        }
    }
    let active: Vec<(u32, f64)> = rate.iter().enumerate().filter(|(_, &r)| r > 0.0).map(|(i, &r)| (i as u32, r)).collect();
    let out = serde_json::json!({
        "trials": trials, "p_release": p_rel, "wall_s": wall,
        "wall_per_bio_s_per_core": wall * rayon::current_num_threads() as f64 / trials as f64,
        "mn9_hz": rate[mn9], "n_active": active.len(),
        "rates": active,
    });
    std::fs::write(&a[3], out.to_string())?;
    println!("trials={trials} wall={wall:.1}s mn9={:.1}Hz activas={} ({:.2} s/s bio por núcleo)",
             rate[mn9], active.len(), wall * rayon::current_num_threads() as f64 / trials as f64);
    Ok(())
}
