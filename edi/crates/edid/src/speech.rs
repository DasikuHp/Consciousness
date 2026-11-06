//! Habla v2: imitar (mimetización) + elegir con el cuerpo fungiforme.
//! - Imitación: ROSA guarda cómo habla la gente; tras un contexto, las continuaciones que
//!   un humano dijo son las candidatas, y la frase se completa copiando su forma de decirlo.
//! - Elección: contexto+candidata se huelen juntos (las KC codifican conjunciones); la
//!   candidata con mejor valencia sináptica (KC→MBON, aprendida con tu aprobación) gana.
//! - Aprendizaje: tu reacción reactiva (replay) la conjunción dicha mientras llega la dopamina.
use edi_rosa::{Rosa, Vocab};

pub const FIN: &str = "<fin>";
pub const MAX_CTX: usize = 8;

/// Olor de una conjunción: 3 glomérulos del contexto (la frase oída) + 3 de la candidata.
pub fn conj_odor(mb: &crate::learn::Mb, ctx: &str, cand: &str) -> Vec<(u32, f32)> {
    let mut o = mb.odor(&format!("CTX:{ctx}"));
    o.extend(mb.odor(&format!("DI:{cand}")));
    o
}

pub fn tokens(v: &mut Vocab, text: &str) -> Vec<u32> {
    text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(|w| v.id(&w.to_lowercase())).collect()
}

/// Candidatas (primeras palabras de respuesta) que un humano dijo tras este contexto.
pub fn candidates(seq: &Rosa, ctx: &[u32], fin: u32, k: usize) -> Vec<u32> {
    let mut c = ctx.to_vec();
    c.push(fin);
    // al menos «palabra <fin>»: un <fin> suelo no es contexto
    seq.continuations_min(&c, MAX_CTX, 2).into_iter().map(|x| x.0).filter(|&t| t != fin).take(k).collect()
}

/// Completa la frase imitando: siguiente palabra más frecuente tras el contexto más largo.
pub fn complete(seq: &Rosa, ctx: &[u32], first: u32, fin: u32, max: usize) -> Vec<u32> {
    let mut c = ctx.to_vec();
    c.push(fin);
    c.push(first);
    let mut out = vec![first];
    while out.len() < max {
        match seq.continuations_min(&c, MAX_CTX, 2).first() { Some(&(t, _)) if t != fin => { out.push(t); c.push(t); } _ => break }
    }
    out
}

/// Pares (pregunta, respuesta) de la memoria de conversación: «q <fin> a <fin> <fin>».
pub fn pairs(seq: &Rosa, fin: u32) -> Vec<(Vec<u32>, Vec<u32>)> {
    let (mut out, mut q, mut cur, mut fins) = (vec![], None::<Vec<u32>>, vec![], 0);
    for &t in &seq.hist {
        if t != fin { cur.push(t); fins = 0; continue; }
        fins += 1;
        if fins >= 2 { q = None; continue; } // separador de pares
        let seg = std::mem::take(&mut cur);
        match q.take() { None => q = Some(seg), Some(qq) => { if !qq.is_empty() && !seg.is_empty() { out.push((qq, seg)); } } }
    }
    out
}

/// Respuestas a frases PARECIDAS (cuando la exacta no está): palabras compartidas con lo que
/// ya oyó, ponderadas por lo raras que son (idf), como cuando un niño contesta «de nada» a
/// «muchas gracias» porque ya oyó «gracias». Devuelve (respuesta completa, parecido), sin repetir
/// primera palabra, de más a menos parecida.
pub fn by_keywords(seq: &Rosa, ctx: &[u32], fin: u32, k: usize) -> Vec<(Vec<u32>, f32)> {
    let ps = pairs(seq, fin);
    if ps.is_empty() { return vec![]; }
    let n = ps.len() as f32;
    let df = |w: u32| ps.iter().filter(|(q, _)| q.contains(&w)).count() as f32;
    let mut uniq: Vec<u32> = ctx.to_vec();
    uniq.sort();
    uniq.dedup();
    let idf: Vec<(u32, f32)> = uniq.iter().map(|&w| { let d = df(w); (w, if d > 0.0 { (1.0 + n / d).ln() } else { 0.0 }) }).collect();
    let mut sc: Vec<(usize, f32)> = ps.iter().enumerate().filter_map(|(i, (q, _))| {
        let s: f32 = idf.iter().filter(|(w, _)| q.contains(w)).map(|x| x.1).sum();
        // normaliza por la longitud de la pregunta (una frase larga comparte palabras por azar)
        (s > 0.0).then(|| (i, s / (q.len() as f32).sqrt()))
    }).collect();
    // a igual parecido, lo más reciente
    sc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(b.0.cmp(&a.0)));
    let mut out: Vec<(Vec<u32>, f32)> = vec![];
    for (i, s) in sc {
        let a = &ps[i].1;
        if out.iter().any(|(o, _)| o[0] == a[0]) { continue; }
        out.push((a.clone(), s));
        if out.len() >= k { break; }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn teach(r: &mut Rosa, v: &mut Vocab, q: &str, a: &str) {
        let fin = v.id(FIN);
        for t in tokens(v, q).into_iter().chain([fin]).chain(tokens(v, a)) { r.push(t); }
        r.push(fin);
        r.push(fin);
    }
    #[test]
    fn pares_y_parecidas() {
        let (mut r, mut v) = (Rosa::new(0), Vocab::default());
        teach(&mut r, &mut v, "hola que tal", "bien y tu");
        teach(&mut r, &mut v, "gracias", "de nada");
        teach(&mut r, &mut v, "como te llamas", "me llamo edi");
        let fin = v.id(FIN);
        assert_eq!(pairs(&r, fin).len(), 3);
        // «muchas gracias» no está, pero comparte «gracias»
        let c = by_keywords(&r, &tokens(&mut v, "muchas gracias"), fin, 3);
        let txt: Vec<String> = c.iter().map(|(a, _)| a.iter().map(|&t| v.label(t).to_string()).collect::<Vec<_>>().join(" ")).collect();
        assert_eq!(txt[0], "de nada");
        // «y tú cómo te llamas» → «me llamo edi» primero
        let c = by_keywords(&r, &tokens(&mut v, "y tu como te llamas"), fin, 3);
        assert_eq!(v.label(c[0].0[0]), "me");
        // nada en común → nada
        assert!(by_keywords(&r, &tokens(&mut v, "patata"), fin, 3).is_empty());
    }
}
