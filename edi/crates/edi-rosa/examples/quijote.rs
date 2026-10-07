//! Acierto de ROSA prediciendo la siguiente letra (online, sin entrenar) frente a un bigrama online.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text: Vec<char> = std::fs::read_to_string(&a[1]).unwrap().chars().collect();
    for n in [2_000usize, 20_000, text.len()] {
        let x: Vec<u32> = text[..n.min(text.len())].iter().map(|&c| c as u32).collect();
        let mut r = edi_rosa::Rosa::new(0);
        let mut big: std::collections::HashMap<u32, std::collections::HashMap<u32, u32>> = Default::default();
        let (mut hr, mut hb, mut hc, mut cov) = (0, 0, 0, 0);
        let t0 = std::time::Instant::now();
        for w in x.windows(2) {
            let p = r.push(w[0]).map(|p| p.token);
            let pb = big.get(&w[0]).and_then(|m| m.iter().max_by_key(|e| e.1).map(|e| *e.0));
            if p == Some(w[1]) { hr += 1; }
            if pb == Some(w[1]) { hb += 1; }
            if p.is_some() { cov += 1; }
            if p.or(pb) == Some(w[1]) { hc += 1; }
            *big.entry(w[0]).or_default().entry(w[1]).or_default() += 1;
        }
        let m = (x.len() - 1) as f64;
        println!("{:>7} letras: ROSA {:.1} % · bigrama {:.1} % · ROSA→bigrama {:.1} % · cobertura {:.0} % · {:.2} µs/token",
            x.len(), 100.0 * hr as f64 / m, 100.0 * hb as f64 / m, 100.0 * hc as f64 / m, 100.0 * cov as f64 / m, t0.elapsed().as_secs_f64() * 1e6 / m);
    }
}
