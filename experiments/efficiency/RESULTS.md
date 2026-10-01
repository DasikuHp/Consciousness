# Eficiencia 1 — motor por eventos (como el cerebro: solo calcula lo que está activo)

Azúcar → MN9, FlyWire v783, 1 s biológico, dt 0,1 ms, 1 núcleo, determinista (`compare_engines`).

| Motor | Tiempo | × tiempo real | Neuronas actualizadas/paso | Spikes | MN9 | Diferencias vs denso |
|---|---|---|---|---|---|---|
| Denso (referencia) | 4,28 s | 0,23 | 138.639 (100 %) | 14.403 | 84 | — |
| Eventos, ε=1e-4 mV | 0,59 s | 1,7 | 10.294 (7,4 %) | 14.403 | 84 | **0** |
| Eventos, ε=1e-3 mV | 0,48 s | 2,1 | 9.397 (6,8 %) | 14.403 | 84 | **0** |
| **Eventos, ε=1e-2 mV (por defecto)** | **0,49 s** | **2,1** | **7.987 (5,8 %)** | **14.403** | **84** | **0** |
| Eventos, ε=5e-2 mV | 0,38 s | 2,6 | 5.488 (4,0 %) | 15.226 | 90 | 294 neuronas (descartado) |

**Funciona:** mismos spikes exactos, ~8,7× menos cómputo. Una neurona en reposo exacto no cambia, así que saltarla no es una aproximación; ε solo decide cuándo una neurona casi en reposo se fija en reposo.
