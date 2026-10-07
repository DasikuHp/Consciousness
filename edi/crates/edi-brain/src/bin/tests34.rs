//! Pruebas 3 y 4 de eficiencia "como un cerebro".
//! 3) Atención multirresolución: simular solo las regiones del foco y medir coste y fidelidad
//!    (azúcar → MN9, con foco en distintas regiones) frente al cerebro completo.
//! 4) Codificación predictiva: un predictor (bigrama aprendido en línea) atenúa la entrada
//!    esperada y deja pasar la sorpresa. Se mide el ahorro de spikes y si la respuesta del
//!    cerebro sigue a la sorpresa (−log p), como la "mismatch negativity" (indicador PP-1).
//! Uso: tests34 <dir_conectoma> <texto>

use edi_brain::{Brain, Connectome, Params};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::time::Instant;

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) { sxy += (a - mx) * (b - my); sxx += (a - mx).powi(2); syy += (b - my).powi(2); }
    sxy / (sxx.sqrt() * syy.sqrt()).max(1e-12)
}

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&a[1]);
    let c = Connectome::load(dir)?;
    let region = std::fs::read(dir.join("region.u8"))?;
    let mn9 = c.group("mn9")[0] as usize;
    let sugar: Vec<(u32, f32)> = c.group("sugar_shiu").iter().map(|&i| (i, 150.0)).collect();

    // ---------- Prueba 3 ----------
    println!("== Prueba 3: atención multirresolución (azúcar → MN9, 1 s, 5 ensayos) ==");
    let names = ["otras", "OL", "MB", "CX", "AL", "LH", "SEZ", "AUD"];
    println!("MN9 está en la región: {}", names[region[mn9] as usize]);
    let cases: Vec<(&str, Option<Vec<u8>>)> = vec![("cerebro completo", None), ("foco SEZ+otras", Some(vec![6, 0])),
        ("foco SEZ", Some(vec![6])), ("foco SEZ+otras+AL+LH", Some(vec![6, 0, 4, 5]))];
    let mut full_counts: Vec<f64> = vec![];
    for (name, regs) in &cases {
        let (mut mn, mut upd, mut steps) = (0u32, 0usize, 0usize);
        let mut counts = vec![0f64; c.n];
        let t0 = Instant::now();
        for trial in 0..5u64 {
            let mut b = Brain::new(&c, Params::default());
            if let Some(r) = regs { b.focus = Some(region.iter().map(|x| r.contains(x)).collect()); }
            let mut rng = ChaCha8Rng::seed_from_u64(100 + trial);
            for _ in 0..10_000 { b.step(&sugar, &mut rng); upd += b.last_updates; steps += 1; }
            mn += b.counts[mn9];
            for (k, &x) in counts.iter_mut().zip(&b.counts) { *k += x as f64; }
        }
        let wall = t0.elapsed().as_secs_f64();
        if full_counts.is_empty() { full_counts = counts.clone(); }
        let r = pearson(&counts, &full_counts);
        println!("{name:22} MN9 {:>5.1} Hz · neuronas/paso {:>6.0} · {:.2} s · correlación con completo {:.3}", mn as f64 / 5.0, upd as f64 / steps as f64, wall, r);
    }

    // ---------- Prueba 4 ----------
    println!("\n== Prueba 4: codificación predictiva (Quijote, 2.000 letras, oído JO) ==");
    let text: Vec<char> = std::fs::read_to_string(&a[2])?.chars().take(2000).collect();
    let mut vocab: Vec<char> = text.clone(); vocab.sort(); vocab.dedup();
    let ids: Vec<usize> = text.iter().map(|ch| vocab.binary_search(ch).unwrap()).collect();
    let v = vocab.len();
    let jo = c.group("auditory_jo").to_vec();
    let mut erng = ChaCha8Rng::seed_from_u64(3);
    let code: Vec<Vec<(u32, f32)>> = (0..v).map(|_| jo.iter().filter(|_| erng.gen::<f32>() < 0.25).map(|&i| (i, 150.0)).collect()).collect();
    for predictive in [false, true] {
        let mut b = Brain::new(&c, Params { dt: 0.5, ..Params::default() });
        b.std_dep = Some((0.3, 100.0));
        b.sfa = Some((2.0, 200.0));
        let mut rng = ChaCha8Rng::seed_from_u64(5);
        let mut bigram = vec![vec![1f64; v]; v];
        let (mut spikes_tot, mut surp, mut resp) = (0u64, vec![], vec![]);
        let mut prev = ids[0];
        for &x in &ids[1..] {
            let row = &bigram[prev];
            let p = row[x] / row.iter().sum::<f64>();
            let gain = if predictive { (1.0 - p) as f32 } else { 1.0 };
            let drive: Vec<(u32, f32)> = code[x].iter().map(|&(i, hz)| (i, hz * gain)).collect();
            b.reset_counts();
            b.run(20.0, &drive, &mut rng);
            let sp: u64 = b.counts.iter().map(|&k| k as u64).sum();
            spikes_tot += sp;
            surp.push(-p.ln());
            resp.push(sp as f64);
            bigram[prev][x] += 1.0;
            prev = x;
        }
        println!("{:24} spikes totales {:>8} · correlación respuesta↔sorpresa {:.3}",
            if predictive { "con predicción (solo sorpresa)" } else { "sin predicción" }, spikes_tot, pearson(&surp, &resp));
    }
    Ok(())
}
