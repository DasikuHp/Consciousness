//! Habla v2, prueba controlada: ¿aprende EDI a contestar como le enseñas?
//! Un profesor humano (guion) habla: cada pregunta con 3 respuestas que EDI le oye (imitación).
//! Luego pregunta; EDI elige con su cuerpo fungiforme y completa la frase imitando. El profesor
//! aprueba solo la respuesta amable (premio → PAM) y desaprueba las otras (castigo → PPL1).
//! Condiciones: conectoma real · barajado · sin plasticidad (azar = 33 %).
//! Uso: speech_test <dir_conectoma> <real|shuffled|frozen> [épocas] [semilla]
#[allow(dead_code)]
#[path = "../learn.rs"] mod learn;
#[allow(dead_code)]
#[path = "../speech.rs"] mod speech;
use edi_brain::{Brain, Connectome, Params};
use edi_rosa::{Rosa, Vocab};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const DIALOG: [(&str, [&str; 3]); 8] = [
    ("hola", ["hola que tal", "adios hasta luego", "no quiero"]),
    ("como estas", ["bien gracias y tu", "fatal", "que te importa"]),
    ("como te llamas", ["me llamo edi", "no lo se", "da igual"]),
    ("quieres jugar", ["si vamos", "ahora no", "jamas"]),
    ("tienes hambre", ["un poco", "nunca", "que dices"]),
    ("te gusta la musica", ["mucho", "odio la musica", "ni idea"]),
    ("buenas noches", ["que descanses", "vete", "ya era hora"]),
    ("gracias", ["de nada", "lo que sea", "por fin"]),
];

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&a[1]);
    let cond = a[2].as_str();
    let epochs: usize = a.get(3).and_then(|x| x.parse().ok()).unwrap_or(10);
    let seed: u64 = a.get(4).and_then(|x| x.parse().ok()).unwrap_or(1);
    let c0 = Connectome::load(dir)?;
    let c = if cond == "shuffled" { c0.shuffled(seed + 100) } else { c0 };
    if cond == "frozen" { std::env::set_var("EDI_ETA", "0"); }
    let mut b = Brain::new(&c, Params { dt: 0.5, p_release: Some(0.5), ..Params::default() });
    b.std_dep = Some((0.3, 100.0)); b.sfa = Some((2.0, 200.0));
    let raw = std::fs::read(dir.join("kc_thr_offset.f32"))?;
    b.thr_offset = Some(raw.chunks_exact(4).map(|x| f32::from_le_bytes(x.try_into().unwrap())).collect());
    let mut mb = learn::Mb::new(&c);
    b.edge_gain = Some(mb.gains(&c));
    let base: Vec<(u32, f32)> = c.group("auditory_jo").iter().map(|&i| (i, 5.0)).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    // 50 ms por tick como edid; olor 8 ticks, descanso 6
    let present = |b: &mut Brain, mb: &mut learn::Mb, od: &[(u32, f32)], da: i32, rng: &mut ChaCha8Rng| -> Vec<f32> {
        let mut acc = vec![];
        for t in 0..14 {
            let mut d = base.clone();
            if t < 8 { d.extend(od.iter().cloned()); }
            if da != 0 && t < 8 { d.extend((if da > 0 { &mb.pam } else { &mb.ppl1 }).iter().map(|&i| (i, learn::DAN_HZ))); }
            b.reset_counts(); b.run(50.0, &d, rng);
            let cnt = std::mem::take(&mut b.counts);
            mb.learn(&cnt, b.edge_gain.as_mut().unwrap());
            if t < 8 { mb.kc_counts(&cnt, &mut acc); }
            b.counts = cnt;
        }
        acc
    };
    // 1) imitación: oye al humano (orden aleatorio)
    let (mut v, mut seq) = (Vocab::default(), Rosa::new(0));
    let fin = v.id(speech::FIN);
    let mut demos: Vec<(usize, usize)> = (0..8).flat_map(|q| (0..3).map(move |r| (q, r))).collect();
    demos.shuffle(&mut rng);
    for (q, r) in demos {
        for t in speech::tokens(&mut v, DIALOG[q].0) { seq.push(t); } seq.push(fin);
        for t in speech::tokens(&mut v, DIALOG[q].1[r]) { seq.push(t); } seq.push(fin);
    }
    // 2) conversación con aprobación
    let mut curve = vec![];
    let mut last_said = vec![];
    for ep in 0..epochs {
        let mut order: Vec<usize> = (0..8).collect(); order.shuffle(&mut rng);
        let mut ok = 0;

        for q in order {
            let ctx = speech::tokens(&mut v, DIALOG[q].0);
            let cands = speech::candidates(&seq, &ctx, fin, 4);
            let mut scored: Vec<(u32, f32, Vec<(u32, f32)>)> = vec![];
            for &cd in &cands {
                let od = speech::conj_odor(&mb, DIALOG[q].0, v.label(cd));
                let acc = present(&mut b, &mut mb, &od, 0, &mut rng);
                let s = mb.valence(&acc, b.edge_gain.as_ref().unwrap());
                mb.seen(&acc);
                scored.push((cd, s, od));
            }
            // exploración pequeña con azar (10 %); si no, la mejor valencia
            let pick = if rng.gen::<f32>() < 0.1 { rng.gen_range(0..scored.len()) } else {
                (0..scored.len()).max_by(|&i, &j| scored[i].1.partial_cmp(&scored[j].1).unwrap()).unwrap() };
            let phrase: Vec<String> = speech::complete(&seq, &ctx, scored[pick].0, fin, 6).iter().map(|&t| v.label(t).to_string()).collect();
            let phrase = phrase.join(" ");
            let good = phrase == DIALOG[q].1[0];
            if good { ok += 1; }
            if ep + 1 == epochs { last_said.push(format!("«{}» → «{}»{}", DIALOG[q].0, phrase, if good { " ✓" } else { "" })); }
            // reacción del profesor: replay de lo dicho + dopamina
            present(&mut b, &mut mb, &scored[pick].2, if good { 1 } else { -1 }, &mut rng);
        }
        curve.push(ok as f64 / 8.0);
        eprintln!("{cond} época {:>2}: {}/8", ep + 1, ok);
    }
    let last3: f64 = curve.iter().rev().take(3).sum::<f64>() / 3.0;
    println!("{cond} semilla {seed}: curva {:?} · media últimas 3 épocas {:.0} %", curve.iter().map(|x| (x * 8.0) as u8).collect::<Vec<_>>(), 100.0 * last3);
    for s in last_said { println!("   {s}"); }
    Ok(())
}
