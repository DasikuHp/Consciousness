//! Compara el motor denso (referencia) con el motor por eventos: mismos spikes y coste.
//! Uso: compare_engines <dir_conectoma> [ms]
use edi_brain::{Brain, Connectome, Params};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let c = Connectome::load(std::path::Path::new(&a[1]))?;
    let ms: f64 = a.get(2).and_then(|x| x.parse().ok()).unwrap_or(1000.0);
    let drive: Vec<(u32, f32)> = c.group("sugar_shiu").iter().map(|&i| (i, 150.0)).collect();
    let mut out = vec![];
    let eps: f32 = a.get(3).and_then(|x| x.parse().ok()).unwrap_or(1e-4);
    for ev in [false, true] {
        let mut b = Brain::new(&c, Params::default());
        b.event_driven = ev;
        b.eps = eps;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let steps = (ms / b.p.dt).round() as usize;
        let (t0, mut upd) = (Instant::now(), 0usize);
        for _ in 0..steps { b.step(&drive, &mut rng); upd += b.last_updates; }
        let wall = t0.elapsed().as_secs_f64();
        println!("{}: {:.2} s para {} ms bio ({:.1}× tiempo real) · neuronas actualizadas/paso = {:.0} ({:.2} %) · spikes = {} · MN9 = {}",
            if ev { "eventos" } else { "denso  " }, wall, ms, ms / 1000.0 / wall, upd as f64 / steps as f64,
            100.0 * upd as f64 / steps as f64 / c.n as f64, b.counts.iter().sum::<u32>(), b.counts[c.group("mn9")[0] as usize]);
        out.push(b.counts.clone());
    }
    let diff = out[0].iter().zip(&out[1]).filter(|(x, y)| x != y).count();
    let tot = out[0].iter().zip(&out[1]).map(|(x, y)| (*x as i64 - *y as i64).abs()).sum::<i64>();
    println!("neuronas con conteo distinto: {diff} · diferencia total de spikes: {tot}");
    Ok(())
}
