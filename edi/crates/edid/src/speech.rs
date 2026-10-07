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
    seq.continuations(&c, MAX_CTX).into_iter().map(|x| x.0).filter(|&t| t != fin).take(k).collect()
}

/// Completa la frase imitando: siguiente palabra más frecuente tras el contexto más largo.
pub fn complete(seq: &Rosa, ctx: &[u32], first: u32, fin: u32, max: usize) -> Vec<u32> {
    let mut c = ctx.to_vec();
    c.push(fin);
    c.push(first);
    let mut out = vec![first];
    while out.len() < max {
        match seq.continuations(&c, MAX_CTX).first() { Some(&(t, _)) if t != fin => { out.push(t); c.push(t); } _ => break }
    }
    out
}
