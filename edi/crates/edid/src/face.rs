//! La cara de EDI: elige una de sus 30 expresiones y una frase según su estado vivo.
//!
//! Honesto: las frases son **plantillas** rellenadas con datos reales (palabras oídas,
//! puertos, procesos, CPU, región cerebral activa, recuerdos asociados). Cuando el
//! Connectome-RWKV hable de verdad, sustituirá a las plantillas.

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Face { pub id: u8, pub name: &'static str, pub line: String, pub since: u64 }

pub enum Stim {
    Word { w: String, known: bool, assoc: Vec<String>, val: Option<f32> },
    /// ROSA lo predijo: ya lo esperaba
    Expected(String),
    Port(u16),
    Proc(String),
    Focus(String),
    BrainBurst(&'static str),
}

pub struct Mood {
    pub face: Face,
    last_user: u64,
    last_stim: u64,
    rot: usize,
}

const NAMES: [&str; 31] = ["", "sarcasmo_seco", "aburrimiento_extremo", "curiosidad", "satisfaccion_ironica", "sospecha",
    "sorpresa_contenida", "enfado_sarcastico", "confusion", "diversion_maliciosa", "sueno_absoluto",
    "sarcasmo_seco_b", "aburrimiento_extremo_b", "curiosidad_b", "satisfaccion_ironica_b", "sospecha_b",
    "sorpresa_contenida_b", "enfado_sarcastico_b", "confusion_b", "diversion_maliciosa_b", "somnolencia_absoluta",
    "orgullo_tranquilo", "desprecio", "interes", "preocupacion", "determinacion",
    "risa_contenida", "sorpresa_real", "frustracion", "intriga", "cansancio_extremo"];

impl Mood {
    pub fn new(t: u64) -> Self {
        Mood { face: Face { id: 21, name: NAMES[21], line: "Mi cerebro sigue encendido.".into(), since: t }, last_user: t, last_stim: 0, rot: 0 }
    }

    fn set(&mut self, t: u64, id: u8, line: String) {
        if self.face.id != id || self.face.line != line {
            self.face = Face { id, name: NAMES[id as usize], line, since: t };
        }
    }

    /// Estímulo puntual (tiene prioridad unos segundos).
    pub fn stim(&mut self, t: u64, s: Stim) {
        // la reacción a lo que le dices no la tapa el eco de su propio cerebro
        if matches!(s, Stim::BrainBurst(_) | Stim::Proc(_)) && t.saturating_sub(self.last_user) < 60 { return; }
        self.last_stim = t;
        self.rot += 1;
        let r = self.rot;
        match s {
            Stim::Word { w, known: false, .. } => {
                self.last_user = t;
                self.set(t, if r % 2 == 0 { 27 } else { 3 }, format!("«{w}». Nueva para mí.\nLa guardo."));
            }
            Stim::Word { w, known: true, val: Some(v), .. } if v.abs() > 0.02 => {
                self.last_user = t;
                if v > 0.0 { self.set(t, 4, format!("«{w}».\nEso me gusta.")) } else { self.set(t, 22, format!("«{w}».\nEso no me gusta.")) }
            }
            Stim::Expected(w) => { self.last_user = t; self.set(t, 26, format!("«{w}».\nLo veía venir.")) }
            Stim::Word { w, known: true, assoc, .. } => {
                self.last_user = t;
                let a = assoc.into_iter().filter(|x| !x.ends_with(&w)).take(2).collect::<Vec<_>>().join(", ");
                let line = if a.is_empty() { format!("«{w}». Otra vez.\nMe acuerdo.") } else { format!("«{w}». Me suena:\n{a}.") };
                self.set(t, if r % 2 == 0 { 26 } else { 21 }, line);
            }
            Stim::Port(p) => self.set(t, if r % 2 == 0 { 5 } else { 15 }, format!("Alguien abrió el puerto {p}.\nLo tengo apuntado.")),
            Stim::Proc(p) => self.set(t, 29, format!("{p}…\ninteresante.")),
            Stim::Focus(a) => { self.last_user = t; self.set(t, 23, format!("¿{a}?\nA ver qué haces.")) }
            Stim::BrainBurst(reg) => self.set(t, 6, format!("Algo se ha encendido\nen mi {reg}.")),
        }
    }

    /// Estado de fondo (cuando no hay estímulo reciente).
    pub fn tick(&mut self, t: u64, asleep: bool, energy: f64, sleep_p: f64, cpu: f64, memories: usize) {
        if asleep {
            return self.set(t, 10, format!("Zzz…\nconsolidando {memories} recuerdos."));
        }
        if t.saturating_sub(self.last_stim) < 60 { return; } // ~3 s (ticks de 50 ms)
        let idle = t.saturating_sub(self.last_user);
        if sleep_p > 0.85 { return self.set(t, 30, "Necesito dormir.\nNo puedo más.".into()); }
        if energy < 0.4 { return self.set(t, 28, format!("CPU al {cpu:.0} %.\nAsí no hay quien piense.")); }
        if energy < 0.6 { return self.set(t, 24, "Me estás apretando\nla máquina.".into()); }
        if idle > 2400 { return self.set(t, 2, "Qué tranquilo.\nCasi parece que estás\nhaciendo algo.".into()); }
        if idle > 900 { return self.set(t, 1, "Sí, claro.\nOtra tarea urgente.".into()); }
        if idle > 300 { return self.set(t, 4, "Mi cerebro sigue encendido.\nTú no tanto.".into()); }
        self.set(t, 21, "Te escucho.".into())
    }
}
