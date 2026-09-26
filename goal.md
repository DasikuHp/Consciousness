# GOAL — Consciousness

> Sustrato neuronal biológico reconstruido (conectoma real de *Drosophila*) + **Laya** como "córtex prefrontal" determinista, para medir hasta dónde llegan la reacción, la decisión, el aprendizaje y los marcadores asociados a la conciencia, y **ver** cómo computan juntos.

Fecha de redacción: 2026-09-26 · Estado: **definición (fase 0 sin empezar)**

---

## 1. Qué es y qué no es

**Es:** un experimento reproducible donde:
1. un conectoma de microscopía electrónica real se ejecuta como red de neuronas *spiking*;
2. recibe estímulos, reacciona y es sometido a tests;
3. se acopla a Laya, que primero **escucha** su estado y después **actúa** sobre él por un canal pequeño y controlado;
4. todo se mide contra controles (Laya apagada, conectoma barajado) y se visualiza en 3D.

**No es:**
- una mosca virtual completa (no se simula el cuerpo en la v1);
- un "animal nuevo" (no se fusionan conectomas de individuos distintos);
- una afirmación de que "hemos creado conciencia". Ninguna métrica actual (PCI, Φ, broadcast…) es una prueba aceptada de conciencia. El adversarial collaboration IIT vs GNWT (Nature, 2025) confirmó y cuestionó predicciones de ambas teorías. Aquí se miden **marcadores**, no se certifica conciencia.

## 2. Pregunta central

> ¿Qué propiedades de percepción, decisión, memoria y estado global aparecen en un conectoma real ejecutado como red spiking y **qué cambia** cuando se acopla a una capa de decisión determinista (Laya), frente a (a) el cerebro solo y (b) un conectoma barajado con la misma distribución de grados?

Si un resultado se reproduce igual con el conectoma barajado, **no se atribuye a la biología**.

## 3. Principios (no negociables)

1. **Dato ≠ hipótesis.** Cada parámetro lleva `source: measured | published_model | hypothesis`. Lo no medido no se rellena con una "mejor suposición" sin marcarlo.
2. **Conectoma inmutable.** La anatomía (quién conecta con quién, cuántas sinapsis, neurotransmisor) es de solo lectura. El aprendizaje vive en una **capa de estado plástico separada** (pesos dinámicos, trazas, ganancias), que se puede resetear al estado 0.
3. **Sinapsis con incertidumbre.** Las sinapsis son predicciones automáticas (en BANC: F1 ≈ 0,83). Se guardan con confianza y se aceptan "no revisadas a mano", pero nunca como verdad absoluta.
4. **Todo se puede apagar.** Laya, la plasticidad, cada módulo: ablación = experimento.
5. **Controles obligatorios.** Todo resultado se compara con: conectoma barajado (preservando grados), Laya apagada y azar.
6. **Determinismo.** Semillas fijas, versiones fijadas y hash de conectoma + parámetros + estado en cada ejecución. Misma entrada ⇒ misma trayectoria.

## 4. Arquitectura

```
                 ┌──────────────── TESTS / ESTÍMULOS ────────────────┐
                 │ sabor · tacto · visión simple · tareas lógicas    │
                 └──────────────────────┬────────────────────────────┘
                                        ▼
   ┌──────────────────────── SUSTRATO (inmutable) ───────────────────────┐
   │ Conectoma: FlyWire v783 → MaleCNS v1.0 → BANC (validación cruzada)  │
   │ neuronas · pares sinápticos · neurotransmisor · confianza · tipo    │
   └──────────────────────┬──────────────────────────────────────────────┘
                          ▼
   ┌──────────── MOTOR NEURONAL ────────────┐   ┌── ESTADO PLÁSTICO ──┐
   │ LIF (Shiu et al.) en toda la red        │◄─►│ ganancias · trazas  │
   │ + modelos más ricos solo donde haya     │   │ regla de plasticidad│
   │   datos (fase posterior)                │   │ (evolucionada)      │
   └──────────────┬─────────────────────────┘   └─────────────────────┘
                  ▼
         ┌──── BUS DE ESTADO ────┐  (resumen comprimido: tasas por tipo/región,
         │  JSON por ventana Δt  │   ensambles activos, salidas motoras/DN)
         └───┬──────────────┬────┘
             ▼              ▼
     ┌── ANÁLISIS ──┐   ┌──────────── LAYA ("córtex prefrontal") ─────────┐
     │ PCIst / LZ   │   │ Fase A: ESCUCHA → clasifica el estado (choice/  │
     │ recurrencia  │   │   score/booleano con probabilidades)            │
     │ persistencia │   │ Fase B: ACTÚA → elige 1 de N acciones de        │
     │ legibilidad  │   │   estimulación sobre un conjunto FIJO y pequeño │
     └──────┬───────┘   │   de neuronas (p. ej., dopaminérgicas del MB)   │
            │           └──────────────────────┬──────────────────────────┘
            ▼                                  │ (canal limitado)
     ┌────────────── DASHBOARD WEB 3D ─────────┴───────────────────────────┐
     │ neuronas en posición real · spikes · métricas · decisiones de Laya  │
     └─────────────────────────────────────────────────────────────────────┘
```

**Regla del canal Laya → cerebro:** Laya **no** puede tocar pesos, conectividad, parámetros ni spikes individuales. Solo puede emitir una acción de un menú cerrado (p. ej., "estimular grupo X a f Hz durante t ms"). Así se sabe dónde termina la biología y dónde empieza Laya.

## 5. Componentes elegidos (verificados que existen)

| Pieza | Uso | Nota realista |
|---|---|---|
| **FlyWire FAFB v783** | Fase 0: reproducir el modelo publicado | ~139k neuronas, solo cerebro |
| **Shiu et al. — `philshiu/Drosophila_brain_model`** (Brian2, MIT) | Motor LIF de referencia | Pensado para v630, con instrucciones para v783; validado frente a experimentos (Nature 2024) |
| **MaleCNS v1.0** (Janelia/Google/Cambridge/MRC-LMB) | **Sustrato principal** a partir de la fase 1 | 166.700 neuronas, 11.710 tipos, cerebro + VNC, totalmente revisado; acceso por neuPrint |
| **BANC** (`htem/BANC-project`) | Segundo animal (hembra), validación cruzada y sinapsis individuales | ~188k neuronas, ~199M sinapsis predichas; **no hay simulador público**, el port es trabajo propio |
| `Kisame76/drosophila-brain-mlx` | Referencia de port a MaleCNS + control barajado | Apple MLX; se usa como referencia de método, no como dependencia |
| `eonsystemspbc/fly-brain` | Referencia de backends (PyTorch/GeNN) | **GPL-2.0**: no copiar código a este repo sin decidir la licencia |
| Base de neurotransmisores de Drosophila (fly connectomics) | Signo y tipo de sinapsis con nivel de evidencia | Sustituye al "GABA/Glu = −1, resto = +1" en fases avanzadas |
| **Laya** (`convaiinnovations/laya`, Apache-2.0) | Córtex prefrontal: escucha y decide | 421M (ModernBERT-large). **No genera texto**: puntúa opciones. Contexto 512 tokens (EN) / hasta 8192 (multilingüe). Viene sobreconfiado → recalibrar temperatura. Zero-shot flojo (0,362 en su benchmark) |
| **RWKV-7 "Goose"** (`BlinkDL/RWKV-LM`) | *Más adelante:* capa que "escucha / imita / habla" | Se sustituye RWKV-v2 (2021) por v7: RNN de estado constante, modelos de 0,1B/0,4B/1,5B/2,9B. Se usa el 0,1B o el 0,4B |
| **PCIst** (Python) | Complejidad perturbacional | Observable global, no "medidor de conciencia" |
| **PyPhi** | IIT, solo en microcircuitos de ≤ ~10 nodos | Φ del cerebro entero es computacionalmente inviable |
| **pycma / ES** | Entrenamiento evolutivo en CPU | Ver §7 |
| FlyVis, FlyGym/NeuroMechFly v2, NEST, NEURON, Arbor | **Aparcados** | Se retoman solo si un test lo requiere (visión rica, cuerpo, biofísica) |

## 6. Rol de Laya

- **Fase A — Escucha (solo lectura).** Laya recibe el resumen JSON del bus de estado y responde preguntas tipadas del estilo "¿qué estímulo recibió?", "¿está en estado de alimentación/aseo/reposo?" o "¿hay estímulo sí/no?".
  - *Test de legibilidad:* acierto de Laya frente al azar, frente a un clasificador lineal simple y frente a la misma tarea con el conectoma barajado.
- **Fase B — Actúa (canal limitado).** Laya elige, de un menú cerrado, una acción de estimulación sobre un grupo fijo de neuronas. Se mide si el sistema **Laya + cerebro** resuelve tareas que el cerebro solo no resuelve, y si esa ventaja desaparece con el conectoma barajado.
- **Fase C (posterior) — RWKV-7.** Capa que recibe la misma interfaz y aprende a "escuchar" el flujo de estados y a producir texto. Solo después de que A y B tengan resultados.

**Limitación a vigilar:** el texto del estado debe caber en el contexto de Laya, así que el bus comprime (tasas por tipo celular o región, no 166k valores).

## 7. Entrenamiento: evolución en CPU, minúscula pero minuciosa

Objetivo: entrenar **desde la nube (sin GPU)** sin que sufra el portátil.

1. **Qué se entrena (el "genoma") — nunca la anatomía:**
   - ganancias/umbrales **por tipo celular**, no por neurona (idea análoga a FlyVis, que ajusta todo el sistema visual con 734 parámetros);
   - **parámetros de la regla de plasticidad** (p. ej., tasa y ventana de una regla dopaminérgica en el cuerpo fungiforme), no los pesos uno a uno;
   - la política de estimulación de Laya en la fase B y su calibración de temperatura.
   - Tamaño objetivo: **10²–10³ parámetros**.
2. **Cómo:** estrategias evolutivas (CMA-ES / OpenAI-ES). Solo necesitan *evaluar* candidatos, sin backprop por la red spiking, y cada evaluación va a un core.
3. **Barato pero minucioso:**
   - **Subcircuitos:** entrenar sobre el subgrafo a k saltos de las entradas y salidas del test (miles de neuronas, no 166k) y **validar después en el cerebro completo**;
   - episodios cortos (cientos de ms biológicos) y *curriculum* de fácil a difícil;
   - **descarte:** los candidatos que fallan los tests lógicos básicos se eliminan antes de las evaluaciones caras;
   - **checkpoints pequeños** (genoma + semilla + hash) guardados en el repo en cada generación, porque el contenedor es efímero y el entrenamiento tiene que poder reanudarse entre sesiones;
   - para Laya, si es viable: cachear los *embeddings* del backbone congelado y ajustar solo la cabeza o la temperatura.
4. **Batería de tests lógicos (de menor a mayor):**
   1. detección (estímulo sí/no);
   2. discriminación A vs. B;
   3. AND / OR;
   4. **XOR**;
   5. retardo con memoria (el estímulo desaparece y la respuesta llega después);
   6. condicionamiento olor + recompensa;
   7. reversión.
5. **Trampa conocida (reservoir computing):** cualquier red recurrente aleatoria con una lectura entrenada resuelve XOR. Por eso **el criterio de éxito no es resolver la tarea, sino resolverla mejor que el conectoma barajado con el mismo presupuesto de entrenamiento**.

## 8. Tests y métricas

| Test | Métrica | "Funciona" si… |
|---|---|---|
| Reproducción Shiu (sabor/tacto → alimentación/aseo) | respuestas de neuronas motoras vs. el paper | se reproducen sus predicciones principales |
| Perturbación | PCIst / Lempel-Ziv, reposo vs. estímulo | diferencia estable entre condiciones y distinta del barajado |
| Recurrencia / persistencia | duración de la actividad tras quitar el estímulo | persistencia medible y reproducible con la misma semilla |
| Legibilidad (Laya A) | accuracy, ECE calibrado | > azar, > barajado, ≥ clasificador lineal |
| Tareas lógicas (evolución) | tasa de éxito por nivel | conectoma real > barajado (test estadístico, varias semillas) |
| Simbiosis (Laya B) | éxito de cerebro + Laya vs. cerebro solo vs. Laya sobre barajado | ganancia atribuible al acoplamiento, no solo a Laya |

## 9. Visualización — dashboard web 3D

- Nube de puntos con la **posición real** del soma de cada neurona (del dataset), coloreada por región o tipo, con spikes que se encienden.
- Paneles de tasas por región, métricas (PCI, persistencia), la entrada y la respuesta de Laya con probabilidades, y la evolución del fitness.
- **Modo reproducción:** la simulación se calcula en la nube y se graba (spikes comprimidos); el navegador solo la reproduce. Así el portátil solo abre una web.
- Tecnología prevista: three.js (WebGL) con datos estáticos precalculados.

## 10. Cómputo y persistencia

| Recurso | Qué corre ahí |
|---|---|
| **Nube (este contenedor):** 4 CPU, 15 GB RAM, **sin GPU**, ~30 GB de disco, **efímero** | descarga/preprocesado de conectomas, LIF en CPU (Brian2 o NumPy/SciPy dispersos), evolución, Laya en CPU, generación de las grabaciones del dashboard |
| **Portátil** (ROG Strix G18, 8 GB VRAM) | solo abrir el dashboard y, **opcionalmente**, ejecuciones ligeras. Nada pesado por defecto |

Límites que condicionan el diseño:
- la tabla de sinapsis individuales de BANC (~199M filas) **no cabe** en 15 GB de RAM, así que se trabaja con pares agregados y se usan sinapsis individuales solo en subcircuitos;
- la velocidad del LIF en CPU está **por medir** en la fase 0 y fija cuántas evaluaciones evolutivas caben por sesión;
- todo artefacto útil (genomas, métricas, grabaciones pequeñas) se hace commit; los datos crudos grandes **no** van a git, sino a un script de descarga reproducible.

## 11. Fases e hitos

| Fase | Entregable | Hecho cuando |
|---|---|---|
| **0 — Base** | Descarga reproducible de FlyWire v783 + LIF de Shiu corriendo en CPU aquí; benchmark de velocidad | se reproducen 2–3 experimentos del paper con semilla fija |
| **1 — Sustrato principal** | Port a MaleCNS v1.0 con formato común (neurona, par, NT, confianza, fuente) + conectoma barajado | mismos tests corren en FlyWire, MaleCNS y barajado |
| **2 — Ver el cerebro** | Dashboard 3D en modo reproducción | se ve un experimento de la fase 0 en el navegador |
| **3 — Laya escucha** | Bus de estado + test de legibilidad | tabla de resultados frente a azar, barajado y lineal |
| **4 — Evolución** | Motor CMA-ES en CPU con checkpoints + batería lógica | curva de fitness real frente a barajado |
| **5 — Laya actúa** | Canal limitado Laya → grupo neuronal + tests de simbiosis | tabla de ablación completa |
| **6 — Marcadores globales** | PCIst/LZ, persistencia; PyPhi en microcircuitos | informe con resultados y límites |
| **7 — BANC y RWKV-7** | Validación cruzada en segundo animal; capa RWKV-7 que escucha | resultados replicados (o no) en BANC |

## 12. Riesgos y límites honestos

- **LIF es una simplificación fuerte.** Sin parámetros biofísicos por neurona, añadir Hodgkin-Huxley "porque sí" sería inventar.
- **Sinapsis predichas** (F1 ≈ 0,83 en BANC): errores estructurales reales.
- **Signo de las sinapsis** inferido del neurotransmisor, sin conocer el receptor concreto.
- **Laya no está hecho para esto:** entiende texto/JSON de negocio, no actividad neuronal. Puede que la legibilidad sea baja; eso también es un resultado.
- **Presupuesto de cómputo:** sin GPU, el cerebro completo solo se evalúa en validación, no en cada generación.
- **Interpretación:** ningún resultado de este proyecto demuestra conciencia. Se reporta como marcadores + controles.

## 13. Fuera de alcance (v1)

Cuerpo biomecánico (FlyGym), visión rica (FlyVis), biofísica multicompartimental (NEURON/Arbor), neuropéptidos con cinética, fusión de conectomas y generación de texto (hasta la fase 7).

## 14. Fuentes

- BANC — https://github.com/htem/BANC-project · https://blog.flywire.ai/2025/11/03/the-banc-brain-and-nerve-cord/
- MaleCNS — https://www.janelia.org/project-team/flyem/male-cns-connectome · https://male-cns.janelia.org/ · https://www.cell.com/cell/fulltext/S0092-8674(26)00942-6
- Shiu et al. LIF — https://github.com/philshiu/Drosophila_brain_model
- fly-brain (multi-backend, GPL-2.0) — https://github.com/eonsystemspbc/fly-brain
- drosophila-brain-mlx — https://github.com/Kisame76/drosophila-brain-mlx
- FlyBrain (navegador) — https://github.com/snedea/flybrain
- Lista curada — https://github.com/cobanov/awesome-fly
- FlyVis — https://github.com/TuragaLab/flyvis
- FlyGym / NeuroMechFly v2 — https://github.com/NeLy-EPFL/flygym
- Laya — https://huggingface.co/convaiinnovations/laya
- RWKV — https://github.com/BlinkDL/RWKV-LM

## 15. Decisiones pendientes

- Licencia del repo (MIT/Apache compatible con Shiu y Laya; evitar mezclar GPL sin decidirlo).
- Formato exacto del bus de estado (qué agregación cabe en el contexto de Laya).
- Grupo neuronal concreto sobre el que actúa Laya en la fase B (candidato: dopaminérgicas PAM/PPL1 del cuerpo fungiforme).
- Si en algún momento se alquila una GPU puntual para acelerar las fases 4–7.
