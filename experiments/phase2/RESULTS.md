# Fase 2 — motor LIF en Rust vs. Brian2 (FlyWire v783, azúcar 150 Hz, 30 ensayos × 1 s)

| | Brian2 (Shiu) | Rust determinista | Rust estocástico (p=0,5) | Rust, conectoma barajado |
|---|---|---|---|---|
| MN9 (Hz) | 77,9 | 94,5 | 93,2 | 0,0 |
| Neuronas activas | 437 | 417 | 471 | 112 |
| CPU por s biológico y núcleo | ~50 s | 5,3 s | 5,5 s | 5,3 s |

- Correlación de Pearson entre las tasas por neurona de Brian2 y Rust: **0,958** (408 neuronas activas en común).
- MN9 sale un 21 % más alto en Rust. Probable causa: orden de actualización dentro del paso (entrada Poisson, retardo y reset). Queda pendiente igualarlo.
- Con el conectoma barajado MN9 no dispara: la respuesta depende del cableado real.
