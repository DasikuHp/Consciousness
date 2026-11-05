//! Conversación con EDI: turnos, respuestas por imitación + cuerpo fungiforme, y tu feedback.
//!
//! Flujo de una frase tuya:
//!  1. `/say` crea tu turno (id) y lo deja en la bandeja; el bucle lo oye (oído JO, olor, SAMN).
//!  2. Si la frase es solo aprobación/desaprobación («bien», «muy mal»…), no es conversación:
//!     es dopamina sobre la última respuesta de EDI.
//!  3. Si no: si contestas a algo que EDI acaba de decir, tu frase queda como respuesta humana
//!     a eso (imitación: así aprende cómo se contesta a «hola que tal»). Y EDI prepara
//!     la suya: candidatas = lo que un humano respondió tras esto (ROSA); si no sabe nada, repite
//!     la palabra más nueva para ella (ecolalia). Huele cada candidata junto al contexto y elige
//!     la de mejor valencia en sus sinapsis KC→MBON; completa la frase imitando.
//!  4. Tu 👍/👎 (aunque llegue tarde) reactiva lo que dijo mientras llega la dopamina (PAM/PPL1).
//!     ✏️ Enseñar: castiga lo que dijo, refuerza la respuesta correcta y la guarda para imitarla.
use crate::{learn::Mb, speech};
use edi_rosa::{Rosa, Vocab};
use serde::Serialize;
use std::collections::VecDeque;

const POS: [&str; 10] = ["bien", "genial", "bravo", "perfecto", "guay", "buena", "bueno", "exacto", "correcto", "muy"];
const NEG: [&str; 9] = ["mal", "malo", "mala", "fatal", "feo", "basta", "error", "incorrecto", "nope"];
const KEEP: usize = 300;

#[derive(Clone, Serialize, serde::Deserialize)]
pub struct Turn {
    pub id: u64,
    pub who: String, // "tú" | "edi"
    pub text: String,
    pub t: u64,
    #[serde(skip_serializing_if = "Option::is_none")] pub reply_to: Option<u64>,
    /// "frase", "feedback", "enseñanza", "imitación", "ecolalia"
    pub kind: String,
    /// candidatas que olió y su valencia (por qué dijo lo que dijo)
    #[serde(skip_serializing_if = "Vec::is_empty", default)] pub cands: Vec<(String, f32)>,
    #[serde(default)] pub reward: i8,
}

#[derive(Default)]
pub struct ChatLog { pub next: u64, pub turns: VecDeque<Turn>, pub thinking: Option<(u64, usize, usize)> }

impl ChatLog {
    pub fn push(&mut self, who: &str, text: &str, kind: &str, reply_to: Option<u64>, cands: Vec<(String, f32)>) -> Turn {
        self.next += 1;
        let t = Turn { id: self.next, who: who.into(), text: text.into(), t: crate::now(), reply_to, kind: kind.into(), cands, reward: 0 };
        self.turns.push_back(t.clone());
        while self.turns.len() > KEEP { self.turns.pop_front(); }
        t
    }
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Turn> { self.turns.iter_mut().rev().find(|t| t.id == id) }
}

pub enum Feedback { Reward { turn: Option<u64>, sign: f32 }, Teach { turn: Option<u64>, text: String } }

/// Sin tildes y en minúsculas, para reconocer palabras de feedback.
pub fn plain(w: &str) -> String {
    w.to_lowercase().chars().map(|c| match c { 'á' => 'a', 'é' => 'e', 'í' => 'i', 'ó' => 'o', 'ú' | 'ü' => 'u', c => c }).collect()
}

/// +1 / −1 si la frase es solo aprobación o desaprobación; None si es conversación.
pub fn feedback_sign(text: &str) -> Option<f32> {
    let ws: Vec<String> = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(plain).collect();
    if ws.is_empty() || ws.len() > 4 { return None; }
    let (p, n) = (ws.iter().filter(|w| POS.contains(&w.as_str())).count(), ws.iter().filter(|w| NEG.contains(&w.as_str())).count());
    if p + n != ws.len() || (p == 1 && n == 0 && ws[0] == "muy") { return None; }
    Some(if n > 0 { -1.0 } else { 1.0 })
}

struct Pending { user_turn: u64, ctx: Vec<u32>, ctx_text: String, cands: Vec<u32>, scores: Vec<Option<f32>>, odors: Vec<Vec<(u32, f32)>>, echo: bool }
struct Said { turn: u64, ctx: Vec<u32>, ctx_text: String, first: u32, odor: Vec<(u32, f32)>, phrase: Vec<u32>, at: u64, answered: bool }

#[derive(Default)]
pub struct Talk {
    pending: Option<Pending>,
    said: VecDeque<Said>,
    /// reactivaciones con dopamina pendientes: (olor, signo)
    pub da_queue: VecDeque<(Vec<(u32, f32)>, f32)>,
    pub n_said: u64,
    pub n_taught: u64,
    pub n_rewards: u64,
}

fn store_pair(talk: &mut Rosa, fin: u32, q: &[u32], a: &[u32]) {
    // «pregunta <fin> respuesta <fin> <fin>»: el doble <fin> separa pares
    for &t in q.iter().chain([fin].iter()).chain(a.iter()) { talk.push(t); }
    talk.push(fin);
    talk.push(fin);
}

impl Talk {
    /// Oye una frase tuya (ya registrada como turno). Devuelve los olores a oler para responder.
    pub fn hear(&mut self, turn: u64, text: &str, reply: bool, talk: &mut Rosa, v: &mut Vocab, mb: &Mb) -> Vec<(String, Vec<(u32, f32)>)> {
        let fin = v.id(speech::FIN);
        let ctx = speech::tokens(v, text);
        if ctx.is_empty() { return vec![]; }
        // imitación: tu respuesta a lo que EDI acaba de decir (≤ 2 min) es una respuesta humana a eso
        if let Some(s) = self.said.back_mut() {
            if !s.answered && crate::now().saturating_sub(s.at) < 120 { store_pair(talk, fin, &s.phrase, &ctx); }
            s.answered = true;
        }
        if !reply { return vec![]; }
        let ctx_text = ctx.iter().map(|&t| v.label(t)).collect::<Vec<_>>().join(" ");
        let mut cands = speech::candidates(talk, &ctx, fin, 4);
        let echo = cands.is_empty();
        if echo {
            // ecolalia: repite la palabra más nueva para ella (la que menos ha oído), como un bebé
            let count = |t: u32| talk.hist.iter().filter(|&&x| x == t).count();
            let w = *ctx.iter().rev().min_by_key(|&&t| count(t)).unwrap();
            cands.push(w);
        }
        let odors: Vec<_> = cands.iter().map(|&c| speech::conj_odor(mb, &ctx_text, v.label(c))).collect();
        let jobs = odors.iter().enumerate().map(|(i, o)| (format!("HABLA?{i}"), o.clone())).collect();
        self.pending = Some(Pending { user_turn: turn, scores: vec![None; cands.len()], ctx, ctx_text, cands, odors, echo });
        jobs
    }

    pub fn thinking(&self) -> Option<(u64, usize, usize)> {
        self.pending.as_ref().map(|p| (p.user_turn, p.scores.iter().filter(|s| s.is_some()).count(), p.scores.len()))
    }

    /// Valencia medida de la candidata i; si ya están todas, elige y devuelve (texto, tipo, candidatas, a qué turno responde).
    pub fn scored(&mut self, i: usize, score: f32, explore: Option<usize>, talk: &Rosa, v: &Vocab) -> Option<(String, String, Vec<(String, f32)>, u64)> {
        let p = self.pending.as_mut()?;
        if i < p.scores.len() { p.scores[i] = Some(score); }
        if p.scores.iter().any(|s| s.is_none()) { return None; }
        let p = self.pending.take()?;
        let sc: Vec<f32> = p.scores.iter().map(|x| x.unwrap()).collect();
        let pick = explore.map(|e| e % sc.len()).unwrap_or_else(|| (0..sc.len()).max_by(|&a, &b| sc[a].partial_cmp(&sc[b]).unwrap()).unwrap());
        let fin = v.get(speech::FIN).unwrap_or(0);
        let phrase = if p.echo { vec![p.cands[pick]] } else { speech::complete(talk, &p.ctx, p.cands[pick], fin, 10) };
        let text = phrase.iter().map(|&t| v.label(t)).collect::<Vec<_>>().join(" ");
        let phrase = phrase.clone();
        let cands = p.cands.iter().zip(&sc).map(|(&c, &s)| (v.label(c).to_string(), (s * 1000.0).round() / 1000.0)).collect();
        let kind = if p.echo { "ecolalia" } else { "imitación" };
        self.said.push_back(Said { turn: 0, ctx: p.ctx, ctx_text: p.ctx_text, first: p.cands[pick], odor: p.odors[pick].clone(), phrase, at: crate::now(), answered: false });
        while self.said.len() > 64 { self.said.pop_front(); }
        self.n_said += 1;
        Some((text, kind.into(), cands, p.user_turn))
    }

    /// El bucle avisa del id del turno de EDI recién creado.
    pub fn set_said_turn(&mut self, id: u64) { if let Some(s) = self.said.back_mut() { s.turn = id; } }

    fn find(&self, turn: Option<u64>) -> Option<&Said> {
        match turn { Some(t) => self.said.iter().rev().find(|s| s.turn == t), None => self.said.back() }
    }

    /// Tu feedback. Devuelve (turno afectado, texto para el registro).
    pub fn feedback(&mut self, fb: Feedback, talk: &mut Rosa, v: &mut Vocab, mb: &Mb) -> Option<(u64, String)> {
        match fb {
            Feedback::Reward { turn, sign } => {
                let s = self.find(turn)?;
                let (id, odor) = (s.turn, s.odor.clone());
                self.da_queue.push_back((odor, sign));
                self.n_rewards += 1;
                Some((id, if sign > 0.0 { "👍".into() } else { "👎".into() }))
            }
            Feedback::Teach { turn, text } => {
                let s = self.find(turn)?;
                let (id, ctx, ctx_text, first, odor) = (s.turn, s.ctx.clone(), s.ctx_text.clone(), s.first, s.odor.clone());
                if let Some(s) = self.said.iter_mut().rev().find(|s| s.turn == id) { s.answered = true; }
                let fin = v.id(speech::FIN);
                let ans = speech::tokens(v, &text);
                if ans.is_empty() { return None; }
                store_pair(talk, fin, &ctx, &ans);
                if ans[0] != first { self.da_queue.push_back((odor, -1.0)); }
                self.da_queue.push_back((speech::conj_odor(mb, &ctx_text, v.label(ans[0])), 1.0));
                self.n_taught += 1;
                Some((id, format!("✏️ {}", ans.iter().map(|&t| v.label(t)).collect::<Vec<_>>().join(" "))))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feedback_solo_si_es_solo_feedback() {
        assert_eq!(feedback_sign("¡muy bien!"), Some(1.0));
        assert_eq!(feedback_sign("Mal"), Some(-1.0));
        assert_eq!(feedback_sign("fatal, muy mal"), Some(-1.0));
        assert_eq!(feedback_sign("estoy bien"), None);
        assert_eq!(feedback_sign("bien gracias y tú"), None);
        assert_eq!(feedback_sign("muy"), None);
        assert_eq!(feedback_sign("Perfécto"), Some(1.0));
        assert_eq!(feedback_sign("gracias"), None); // se contesta («de nada»), no es premio
    }
}
