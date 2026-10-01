//! Diagnóstico: ¿cómo se propaga un olor débil por el conectoma? Actividad por región en ventanas de 50 ms.
use edi_brain::{Brain, Connectome, Params};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&a[1]);
    let c = Connectome::load(dir)?;
    let region = std::fs::read(dir.join("region.u8"))?;
    let hz: f32 = a.get(2).and_then(|x| x.parse().ok()).unwrap_or(5.0);
    let grp = a.get(3).cloned().unwrap_or("orn_da1".into());
    let kc = c.group("kenyon_cell").to_vec();
    let mut b = Brain::new(&c, Params { dt: 0.25, ..Params::default() });
    if let Some(t) = a.get(4).and_then(|x| x.parse::<f32>().ok()) { b.std_dep = Some((0.5, t)); }
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let drive: Vec<(u32, f32)> = c.group(&grp).iter().map(|&i| (i, hz)).collect();
    let names = ["otras", "OL", "MB", "CX", "AL", "LH", "SEZ", "AUD"];
    for w in 0..10 {
        b.reset_counts();
        b.run(50.0, if w < 6 { &drive } else { &[] }, &mut rng);
        let mut act = [0usize; 8];
        for (i, &k) in b.counts.iter().enumerate() { if k > 0 { act[region[i] as usize % 8] += 1; } }
        let kca = kc.iter().filter(|&&k| b.counts[k as usize] > 0).count();
        println!("t={:>3}-{:>3} ms {} · KC {kca} · {}", w * 50, w * 50 + 50, if w < 6 { "olor" } else { "    " },
            (0..8).map(|k| format!("{} {}", names[k], act[k])).collect::<Vec<_>>().join(" · "));
    }
    Ok(())
}
