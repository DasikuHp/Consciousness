//! Fuente de azar de EDI: futuro abierto, pasado reproducible.
//!
//! El azar físico (SO: RDRAND/jitter vía getrandom) o cuántico (bytes de ANU QRNG
//! guardados en un pool) se usa como *semilla* de ChaCha8 por época. Cada semilla
//! se escribe en una cinta (tape); reproducir la cinta revive exactamente el pasado.

use anyhow::{bail, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

pub enum Source {
    /// Pseudoaleatorio fijo (experimentos reproducibles, validación).
    Fixed(u64),
    /// Entropía física del sistema operativo.
    Os,
    /// Bytes cuánticos precargados (p. ej. ANU QRNG); se consumen en orden.
    Quantum(PathBuf),
    /// Revive una cinta grabada.
    Replay(PathBuf),
}

pub struct Entropy {
    src: Source,
    tape: Option<std::fs::File>,
    cursor: u64,
    epoch: u64,
}

impl Entropy {
    pub fn new(src: Source, tape: Option<PathBuf>) -> Result<Self> {
        let tape = match tape {
            Some(p) => Some(OpenOptions::new().create(true).append(true).open(p)?),
            None => None,
        };
        Ok(Entropy { src, tape, cursor: 0, epoch: 0 })
    }

    fn read_at(path: &PathBuf, off: u64, buf: &mut [u8]) -> Result<()> {
        use std::io::{Seek, SeekFrom};
        let mut f = std::fs::File::open(path)?;
        f.seek(SeekFrom::Start(off))?;
        if f.read(buf)? != buf.len() {
            bail!("sin bytes suficientes en {}", path.display());
        }
        Ok(())
    }

    /// Nueva semilla de 32 bytes para la siguiente época.
    pub fn seed(&mut self) -> Result<[u8; 32]> {
        let mut s = [0u8; 32];
        match &self.src {
            Source::Fixed(x) => {
                s[..8].copy_from_slice(&x.to_le_bytes());
                s[8..16].copy_from_slice(&self.epoch.to_le_bytes());
            }
            Source::Os => getrandom::getrandom(&mut s)?,
            Source::Quantum(p) | Source::Replay(p) => {
                let p = p.clone();
                Self::read_at(&p, self.cursor, &mut s)?;
                self.cursor += 32;
            }
        }
        if let Some(t) = &mut self.tape {
            t.write_all(&s)?;
        }
        self.epoch += 1;
        Ok(s)
    }

    pub fn rng(&mut self) -> Result<ChaCha8Rng> {
        Ok(ChaCha8Rng::from_seed(self.seed()?))
    }
}
