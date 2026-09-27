# Fase 0: reproducción de Shiu et al. 2024 (azúcar → MN9)

Montaje: LIF de `external/fly/Drosophila_brain_model` (Brian2 2.9, NumPy 2.3). 21 GRN de azúcar estimuladas por Poisson a 150 Hz, 5 ensayos de 1 s biológico, en 4 CPU sin GPU.

| Conectoma | MN9 (Hz) | Neuronas activas | Tiempo real por s biológico |
|---|---|---|---|
| FlyWire v630 (el del paper) | 83,2 | 421 | 7,9 s |
| FlyWire v783 (20/21 GRN con el mismo ID) | 79,0 | 400 | 18,7 s* |

\* La ejecución v783 coincidió con compresión y push de git, así que el tiempo está inflado.

**Conclusión:** se reproduce cualitativamente el resultado del paper: la activación de sensores de azúcar hace disparar la motoneurona de la probóscide MN9. El pipeline funciona en esta nube.

**Pendiente:** control con conectoma barajado preservando grados, más ensayos (el paper usa 30) y curva de frecuencia de estímulo.
