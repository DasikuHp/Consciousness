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

## Integración en EDI (`edid`) + ROSA de RWKV-8 (2026-10-07)

**Ya vivo en `edid`:** motor por eventos, agotamiento sináptico (U=0,3, τ=100 ms), adaptación (Δθ=2 mV, 200 ms), umbrales homeostáticos de KC (`kc_thr_offset.f32`), dopamina moduladora y plasticidad de 3 factores KC→MBON con traza de elegibilidad, persistida en `kc_mbon.f32`. El bucle va a tiempo real (antes corría sin freno a ~40×).

**Conceptos como olores.** Cada palabra, ventana o puerto activa 3 tipos reales de ORN (de 53), elegidos por hash, a 120 Hz durante 400 ms. El refuerzo llega por "bien/genial/gracias…" o `POST /reward +` (PAM) y por "mal/malo/basta…" o `POST /reward -` (PPL1).

**Valencia.** Se lee de las sinapsis KC→MBON que activa el concepto: la depresión hacia los MBON de compartimento PAM menos la de PPL1. Antes se resta lo inespecífico: la media del compartimento y la actividad media de KC (como la inhibición APL). Cada MBON va al compartimento de su DAN dominante por nº de sinapsis; el etiquetado bruto solapaba 57 de 68.

Offline, determinista (`valence_test`, misma configuración que edid). 6 ensayos: café+premio, lluvia+castigo, mesa sin refuerzo.

| semilla | café | lluvia | mesa (control) |
|---|---|---|---|
| 1 | **+0,021** | −0,069 | −0,069 |
| 2 | **+0,027** | **−0,076** | −0,037 |
| 3 | **+0,026** | **−0,048** | −0,005 |

En vivo, por HTTP: café +0,030, lluvia −0,047, mesa −0,014, silla +0,016, luz 0,000. Diciendo "lluvia", EDI pone cara de desprecio y responde «Eso no me gusta»; la valencia sobrevive al reinicio.

**Honesto:**
- El premio se aprende de forma específica (3/3 semillas). El castigo, solo en 2/3: en la semilla 1, lluvia y el control empatan.
- Efecto pequeño (~0,03–0,07 de depresión específica). Errores previos que lo hacían inespecífico: saturación, solape de compartimentos, el oído común a todas las palabras y procesos del propio test que EDI también olía y reforzaba.

**ROSA (RWKV-8, autómata de sufijos online).** Port a Rust (`edi-rosa`), idéntico a `rosa()` de `251014_rosa_1bit_layer.py` en 300/300 secuencias aleatorias.
- Predicción de la siguiente letra del Quijote, sin entrenar ni pesos: 2.000 letras → 34,4 % (bigrama 29,8 %); 29.376 → 46,1 % (30,3 %); 2,17 M → **56,3 %** (30,6 %), a 2 µs/token. El reservorio del conectoma daba 33,9 % entrenado.
- En EDI:
  - (1) memoria episódica exacta de la secuencia de conceptos: `/recall?q=hola` → «después: edi, que, tal»;
  - (2) codificación predictiva: lo que ROSA predijo entra atenuado al 30 % y EDI dice «Lo veía venir»; acierto en la sesión de prueba: 55 %;
  - (3) ROSA de 1 bit por región cerebral (¿sube o baja su actividad?): acierto 69–77 % → "predictibilidad" del cerebro.

**Coste:** reposo 4 % de un núcleo, ~520 neuronas/paso. Mientras oye y huele palabras, ~100 % (hasta 30.000 neuronas/paso). Quitar el lóbulo óptico del foco no ahorra nada en vivo: sin estímulo visual ya está en silencio y el motor por eventos lo salta.
