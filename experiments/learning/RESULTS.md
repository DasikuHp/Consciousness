# Prueba 2 — aprendizaje dopaminérgico real (condicionamiento olfativo)

Cuerpo fungiforme de FlyWire v783 · olor A = ORN DA1+VA1d+DL3 · olor B = ORN VA1v+VL1+VM4 · 60 Hz · 6 ensayos A+dopamina (optogenética in silico, 331 DAN a 150 Hz), B sin dopamina · regla de tres factores en Kenyon→MBON: g ← g·(1 − η·spikes_KC·dopamina_MBON), donde la dopamina de cada MBON proviene de las DAN que la inervan **en el conectoma**.

## Lo que se descubrió por el camino (cada paso probado)

| Paso | Observación | Mecanismo añadido (hipótesis de parámetros, mecanismo real) |
|---|---|---|
| LIF puro (Shiu) | Un olor débil (5 Hz) provoca **ignición**: ~9.000 neuronas encendidas que siguen activas tras el olor; 75 % de KC activas con cualquier olor | — |
| + depresión sináptica a corto plazo (Tsodyks–Markram, U=0,3, τ=100 ms) | Desaparece la ignición, pero queda actividad autosostenida en reposo (~18 % neuronas) que bloquea olores siguientes | STD presináptica perezosa (coste ~0) |
| + adaptación de frecuencia (Δθ=2 mV, τ=200 ms) | El cerebro **vuelve al reposo** tras cada olor; KC dispersas (~5 %), pero A y B activan las mismas KC (Jaccard 0,80) | Umbral adaptativo perezoso |
| + plasticidad homeostática intrínseca de KC (objetivo 5 %) | **Jaccard 0,80 → 0,10**: código disperso y específico de olor, emergente de una regla local | Calibración con 100 olores aleatorios de 53 glomérulos reales |
| Dopamina como excitación rápida (modelo base) | La estimulación DAN activa casi todas las KC → la memoria no es específica | — |
| + dopamina **neuromoduladora** (sin corriente rápida, solo tercer factor) | Memoria específica (abajo) | Fisiología conocida (receptores acoplados a proteína G) |

## Resultado (corriente sináptica KC→MBON, media de 4 ensayos de prueba)

| Condición | A antes → después | B antes → después | Índice de memoria (ratio_B − ratio_A) |
|---|---|---|---|
| Real, semilla 1 | 7.440 → 1.156 | 5.180 → 3.287 | **+0,479** |
| Real, semilla 2 | 7.547 → 1.232 | 5.063 → 2.535 | **+0,337** |
| Real, semilla 3 | 7.639 → 1.593 | 5.821 → 2.595 | **+0,237** |
| Real sin plasticidad (control) | ±10 % | ±10 % | +0,16 / −0,00 / −0,25 (media −0,03) |
| Conectoma barajado | 0 → 0 | 0 → 0 | — (el olor no llega a las KC) |

**Funciona (preliminar):** el conectoma real aprende una memoria específica del olor emparejado; sin plasticidad no hay efecto; con el cableado barajado el sistema ni siquiera puede representar el olor. Límites: 3 semillas; B también se deprime parcialmente (generalización); las MBON apenas disparan, por eso se mide la corriente (como la imagen de calcio). Parámetros de STD/SFA/homeostasis son hipótesis.
