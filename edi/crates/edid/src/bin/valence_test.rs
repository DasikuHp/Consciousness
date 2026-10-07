//! Condicionamiento de conceptos con la configuración exacta de edid (sin HTTP, determinista):
//! café+premio (PAM), lluvia+castigo (PPL1), mesa sin refuerzo. Mide solapamiento de KC y valencia.
//! Uso: valence_test <dir_conectoma> [hz_orn] [norm_dan 0|1]
#[allow(dead_code)]
#[path = "../learn.rs"] mod learn;
use edi_brain::{Brain, Connectome, Params};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&a[1]);
    let c = Connectome::load(dir)?;
    let mut b = Brain::new(&c, Params { dt: 0.5, p_release: Some(0.5), ..Params::default() });
    b.std_dep = Some((0.3, 100.0)); b.sfa = Some((2.0, 200.0));
    let raw = std::fs::read(dir.join("kc_thr_offset.f32"))?;
    b.thr_offset = Some(raw.chunks_exact(4).map(|x| f32::from_le_bytes(x.try_into().unwrap())).collect());
    let region = std::fs::read(dir.join("region.u8"))?;
    b.focus = Some(region.iter().map(|&r| r != 1).collect());
    let mut mb = learn::Mb::new(&c);
    b.edge_gain = Some(mb.gains(&c));
    let base: Vec<(u32, f32)> = c.group("auditory_jo").iter().map(|&i| (i, 5.0)).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    for (name, drv) in [("reposo", 0), ("PAM 150 Hz", 1), ("PPL1 150 Hz", -1)] {
        let mut d = base.clone();
        if drv != 0 { d.extend((if drv > 0 { &mb.pam } else { &mb.ppl1 }).iter().map(|&i| (i, learn::DAN_HZ))); }
        b.reset_counts(); b.run(300.0, &d, &mut rng);
        let sp = |v: &[u32]| v.iter().map(|&i| b.counts[i as usize]).sum::<u32>() as f32 / v.len() as f32 / 0.3;
        println!("{name:12}: PAM {:.1} Hz/neurona ({}) · PPL1 {:.1} Hz/neurona ({})", sp(&mb.pam), mb.pam.len(), sp(&mb.ppl1), mb.ppl1.len());
    }
    let words = ["OYE:cafe", "OYE:lluvia", "OYE:mesa"];
    // ticks de 50 ms como en edid; dopamina opcional durante el olor
    let da_from: i32 = std::env::var("DA_FROM").ok().and_then(|x| x.parse().ok()).unwrap_or(0);
    let present = |b: &mut Brain, mb: &mut learn::Mb, w: &str, da: i32, rng: &mut ChaCha8Rng| -> Vec<f32> {
        let mut od = mb.odor(w);
        if std::env::var("JO").as_deref() != Ok("0") { // como en edid: la palabra también se oye (24 JO a 150 Hz)
            let jo = c.group("auditory_jo"); let h = learn::fnv(&w[4..]);
            od.extend((0..24u64).map(|k| (jo[((h >> 3).wrapping_add(k * 2654435761) as usize) % jo.len()], 150.0)));
        }
        let mut acc = vec![];
        for t in 0..28 {
            let mut d = base.clone();
            if t < 8 { d.extend(od.iter().cloned()); }
            if da != 0 && (da_from..8).contains(&t) { d.extend((if da > 0 { &mb.pam } else { &mb.ppl1 }).iter().map(|&i| (i, learn::DAN_HZ))); }
            b.reset_counts(); b.run(50.0, &d, rng);
            let cnt = std::mem::take(&mut b.counts);
            mb.learn(&cnt, b.edge_gain.as_mut().unwrap());
            if t < 8 { mb.kc_counts(&cnt, &mut acc); }
            b.counts = cnt;
        }
        mb.seen(&acc);
        acc
    };
    println!("MBON con PAM {} · con PPL1 {} · ambos {}", mb.n_dan_mbon().0, mb.n_dan_mbon().1, mb.n_dan_mbon().2);
    for w in ["OYE:silla", "OYE:luz", "OYE:perro", "OYE:hola"] { present(&mut b, &mut mb, w, 0, &mut rng); }
    let sets: Vec<Vec<f32>> = words.iter().map(|w| present(&mut b, &mut mb, w, 0, &mut rng)).collect();
    let on = |v: &[f32]| v.iter().enumerate().filter(|x| *x.1 > 0.0).map(|x| x.0).collect::<std::collections::HashSet<_>>();
    for i in 0..3 { for j in i + 1..3 {
        let (x, y) = (on(&sets[i]), on(&sets[j]));
        println!("KC {} {} vs {} {} · Jaccard {:.2}", words[i], x.len(), words[j], y.len(), x.intersection(&y).count() as f64 / x.union(&y).count().max(1) as f64);
    } }
    for _ in 0..6 { for (w, da) in [(words[0], 1), (words[1], -1), (words[2], 0)] { present(&mut b, &mut mb, w, da, &mut rng); } }
    for w in words { let acc = present(&mut b, &mut mb, w, 0, &mut rng); let (p, q) = mb.valence2(&acc, b.edge_gain.as_ref().unwrap()); println!("{w:12} valencia {:+.3} (dep. PAM {p:.3} · PPL1 {q:.3})", p - q); }
    println!("sinapsis deprimidas {}", mb.depressed(b.edge_gain.as_ref().unwrap()));
    Ok(())
}
