//! Diagnóstico: olor A → descanso → olor B en el mismo cerebro (con STD y sin KC→KC).
use edi_brain::{Brain, Connectome, Params};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;
fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let c = Connectome::load(std::path::Path::new(&a[1]))?;
    let kc: HashSet<u32> = c.group("kenyon_cell").iter().cloned().collect();
    let mut gain = vec![1.0f32; c.meta.n_edges];
    for &k in &kc { for e in c.row_ptr[k as usize] as usize..c.row_ptr[k as usize + 1] as usize { if kc.contains(&c.col[e]) { gain[e] = 0.0; } } }
    let mut b = Brain::new(&c, Params { dt: 0.25, ..Params::default() });
    b.edge_gain = Some(gain);
    b.std_dep = Some((0.3, 100.0));
    if let Some(d) = a.get(2).and_then(|x| x.parse::<f32>().ok()) { b.sfa = Some((d, a.get(3).and_then(|x| x.parse().ok()).unwrap_or(200.0))); }
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let od = |names: &[&str]| names.iter().flat_map(|n| c.group(n).to_vec()).map(|i| (i, 60.0f32)).collect::<Vec<_>>();
    let (oa, ob) = (od(&["orn_da1", "orn_va1d", "orn_dl3"]), od(&["orn_va1v", "orn_vl1", "orn_vm4"]));
    for (name, d, ms) in [("A", &oa, 300.0), ("reposo", &vec![], 1500.0), ("B", &ob, 300.0), ("reposo", &vec![], 1500.0), ("A", &oa, 300.0)] {
        b.reset_counts();
        b.run(ms, d, &mut rng);
        let kca = kc.iter().filter(|&&k| b.counts[k as usize] > 0).count();
        let orn = d.iter().filter(|(i, _)| b.counts[*i as usize] > 0).count();
        println!("{name:7} {ms:>5} ms · ORN que disparan {orn}/{} · KC {kca} · spikes {} · activas ahora {:.2} %", d.len(), b.counts.iter().sum::<u32>(), 100.0 * b.active_fraction());
    }
    Ok(())
}
