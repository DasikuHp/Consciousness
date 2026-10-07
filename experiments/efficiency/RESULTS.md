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

# Eficiencia 3 — atención multirresolución (`tests34`)
Azúcar → MN9 (MN9 está en SEZ), 1 s × 5 ensayos. Fuera del foco no se calcula nada.

| Foco | MN9 | Neuronas/paso | Tiempo | Correlación de conteos con el cerebro completo |
|---|---|---|---|---|
| Cerebro completo | 96,6 Hz | 8.065 | 2,73 s | 1,000 |
| SEZ + otras | 93,8 Hz | 5.704 | 1,76 s | 1,000 |
| **Solo SEZ** | **97,0 Hz** | **3.122** | **1,05 s** | **0,999** |

**Funciona:** atendiendo solo la región relevante, el comportamiento (MN9) se conserva y el coste baja 2,6×. Límite: aquí el foco se elige a mano; falta que EDI lo elija (workspace/atención).

# Eficiencia 4 — codificación predictiva
Quijote, 2.000 letras al oído (JO), con STD+SFA. Un predictor de bigramas aprendido en línea atenúa la entrada esperada.

| | Spikes totales | Correlación respuesta ↔ sorpresa (−log p) |
|---|---|---|
| Sin predicción | 511.748 | −0,07 |
| **Con predicción** | **462.998 (−9,5 %)** | **+0,62** |

**Funciona en parte:** aparece una señal de sorpresa clara (indicador PP-1, análoga a la "mismatch negativity"); el ahorro de energía es modesto (−9,5 %) porque la mayor parte de la actividad no es la entrada sino su propagación interna. El predictor es externo (bigrama); el siguiente paso es que la predicción la haga el propio cerebro.
