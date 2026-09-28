//! Simbiosis con el escritorio: el estado de EDI colorea Niri (recarga en vivo el include).
use std::path::Path;

pub fn color(asleep: bool, energy: f64) -> (&'static str, &'static str) {
    if asleep { ("#6b46c1", "#241a3d") }
    else if energy < 0.5 { ("#f6ad55", "#3d2c14") }
    else { ("#4fd1c5", "#16302e") }
}

pub fn write(dir: &Path, asleep: bool, energy: f64) -> std::io::Result<bool> {
    let (act, inact) = color(asleep, energy);
    let body = format!("// gestionado por edid: el desconcierto de EDI colorea el escritorio\nlayout {{\n    focus-ring {{\n        active-color \"{act}\"\n        inactive-color \"{inact}\"\n    }}\n}}\n");
    let p = dir.join("edi-mood.kdl");
    if std::fs::read_to_string(&p).map(|s| s == body).unwrap_or(false) { return Ok(false); }
    let tmp = dir.join(".edi-mood.tmp");
    std::fs::write(&tmp, body)?;
    std::fs::rename(tmp, p)?;
    Ok(true)
}
