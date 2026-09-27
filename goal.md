# GOAL — Consciousness / BrainOS

> Un **sistema operativo real** cuyo "usuario" es un **cerebro biológico reconstruido** (conectoma de *Drosophila* ejecutado como red spiking), con **Laya** como córtex prefrontal determinista y **RWKV-7** como voz que aprende sobre la marcha. Vive dentro de un **sandbox (neko)** donde ve la pantalla, mueve el ratón, teclea y te escribe notas: primero en **modo fantasma** y después, cuando demuestra que sabe lo que hace y tú lo autorizas, **de forma autónoma**. Todo medido, reproducible y visible en 3D.

Última revisión: 2026-09-27 · Rama: `claude/bold-dirac-s0e3t4`
Estado: fase 0 ✅ (Shiu reproducido en CPU); datos, pesos y código descargados en el repo.

---

## 1. Qué queremos de verdad

1. **Simular computación neuronal con base biológica fiel:** conectoma EM real → spikes → comportamiento. Nada inventado sin marcarlo.
2. **Medir marcadores asociados a la conciencia** (integración, recurrencia, persistencia, complejidad perturbacional, legibilidad del estado), con controles. **No** afirmamos crear conciencia; construimos el sustrato para poner hipótesis a prueba.
3. **Simbiosis cerebro ⇄ Laya:** Laya escucha el cerebro y luego actúa sobre él por un canal limitado, como un córtex prefrontal.
4. **BrainOS:** un S.O. real (arrancable en nube, VM y USB) con todo el userland en **Rust**, salvo el cerebro.
5. **Cuerpo digital:** el cerebro vive en un escritorio sandbox (neko). Ve, mueve el ratón, teclea y escribe notas.
6. **Autonomía por escalera:** fantasma → validado → autónomo, con métrica **y** tu permiso.
7. **Aprender sobre la marcha** con un sistema de entrenamiento innovador y barato (sin GPU): **MEM-EGGROLL** (§8).

## 2. Principios (no negociables)

1. **Dato ≠ hipótesis.** Todo parámetro lleva `source: measured | published_model | hypothesis`.
2. **Conectoma inmutable.** El aprendizaje vive en capas separadas (estado plástico, adaptadores low-rank, memoria), que se pueden resetear.
3. **Todo se puede apagar** (Laya, RWKV, memoria, plasticidad): cada ablación es un experimento.
4. **Controles obligatorios:** conectoma barajado (preservando grados), Laya apagada y azar. Si el conectoma barajado rinde igual, no se atribuye a la biología.
5. **Determinismo:** semillas, versiones fijadas, hash de conectoma + parámetros + estado. Misma entrada ⇒ misma trayectoria.
6. **Seguridad del sandbox:** el cerebro **solo** actúa dentro de neko, nunca sobre el host. Siempre hay un botón de "volver a fantasma" y otro de "matar".
7. **Honestidad:** se reporta lo que falla igual que lo que funciona.

## 3. Arquitectura

```
╔══════════════════════════ BrainOS (Arch + Okimarchy/Niri) ═══════════════════════════╗
║                                                                                       ║
║   ┌──────────── SANDBOX: neko (Docker, WebRTC) ─────────────┐   tú (navegador)        ║
║   │  escritorio virtual: navegador, editor, apps de test    │◄──── miras / interactúas ║
║   └───────▲───────────────────────────────┬────────────────┘                          ║
║           │ ratón/teclado (xdotool)        │ píxeles (captura X11)                      ║
║   ┌───────┴──────┐                 ┌───────▼───────┐                                   ║
║   │ motor  [Rust]│                 │ retina [Rust] │ píxeles → tasas de fotorreceptores ║
║   └───────▲──────┘                 └───────┬───────┘                                   ║
║   ┌───────┴───────┐   intención    ┌───────▼────────────────────────────┐              ║
║   │ ghost  [Rust] │◄───────────────│ CEREBRO [Python/Brian2 → GPU opc.] │              ║
║   │ overlay:      │                │ conectoma real (FlyWire/MaleCNS/   │              ║
║   │ fantasma /    │                │ BANC), LIF, neurotransmisores      │              ║
║   │ validado /    │                └───────┬──────────────▲─────────────┘              ║
║   │ autónomo      │                        │ bus de estado │ estimulación limitada     ║
║   └───────▲───────┘                ┌───────▼──────────────┴─────────────┐              ║
║           │ ¿válido? p=0.93        │ bus  [Rust]  (estado comprimido)   │              ║
║   ┌───────┴────────┐               └───┬───────────┬──────────────┬─────┘              ║
║   │ cortex [Rust]  │◄──────────────────┘           │              │                    ║
║   │ Laya (candle/  │                      ┌────────▼─────┐ ┌──────▼──────────┐          ║
║   │ ONNX): decide  │                      │ voice [Rust] │ │ memory [Rust]   │          ║
║   └────────────────┘                      │ RWKV-7 notas │ │ grafo episódico │          ║
║                                           │ + SEAL/EGG   │ │ semántico/proc. │          ║
║                                           └──────────────┘ └─────────────────┘          ║
║   ┌──────────────────────────── dashboard [Rust + WebGL] ───────────────────────────┐  ║
║   │ cerebro 3D en directo · métricas · decisiones de Laya · notas · autonomía        │  ║
║   └──────────────────────────────────────────────────────────────────────────────────┘  ║
╚═══════════════════════════════════════════════════════════════════════════════════════╝
```

## 4. Componentes y decisiones

| Pieza | Decisión | Por qué / nota realista |
|---|---|---|
| **Cerebro** | Python + Brian2 (LIF de Shiu et al.), luego port a MaleCNS y BANC | Es "el cerebro de verdad" y **no** se reescribe en Rust. Se puede acelerar con GeNN/PyTorch si hay GPU |
| **Conectomas** | FlyWire v783 (validación) → **MaleCNS v1.0** (principal) → **BANC** (segundo animal) | Ya están en `vault/`. Sinapsis individuales solo en subcircuitos (no caben en RAM) |
| **Humano / otros** | H01 (neuronas humanas reales), TVB (conectomas humano, macaco y ratón; PCI en TVB-AdEx), C. elegans y larva completos como controles | No existe cerebro humano completo a nivel sináptico: lo humano entra como modelos de neurona y escala global |
| **Laya** (córtex prefrontal) | Inferencia desde Rust (candle u ONNX Runtime) | Decide y valida (`choice`/`score`/booleano con probabilidad). **No genera texto**. Recalibrar temperatura |
| **RWKV-7** (voz) | G1d 0,1B (en `vault/`), inferencia en Rust con `candle-rwkv` (CPU) o `web-rwkv` (GPU) | RNN de estado constante. EGGROLL ya se demostró sobre RWKV-7 |
| **Base del S.O.** | **Arch Linux + Okimarchy** (Omarchy con **Niri**, compositor Wayland en Rust) | Un kernel propio en Rust no ejecutaría Docker, neko ni Python |
| **Userland propio** | Todo en **Rust**: `retina`, `motor`, `ghost`, `bus`, `cortex`, `voice`, `memory`, `dashboard`, `brainosd` (supervisor) | Workspace Cargo en `brainos/` |
| **Sandbox** | **neko** (m1k1o) en Docker; el cerebro solo toca este escritorio | Aislamiento. Tú lo ves y controlas por WebRTC |
| **Visualización** | Dashboard WebGL con neuronas en su posición real y spikes en directo | Somas y posiciones en los datos de BANC, FlyWire y MaleCNS |

## 5. El cuerpo digital: de píxeles a spikes y de spikes a ratón

- **Entrada (retina):** captura de pantalla de neko → reducción a una rejilla hexagonal tipo omatidios → tasas de Poisson a los fotorreceptores R1–R8 del conectoma. Opcional: FlyVis como front-end visual validado.
- **Salida (motor):** tasas de neuronas descendentes y motoras seleccionadas → decodificador → `dx, dy`, clic, tecla.
  - El mapeo neurona→acción es **hipótesis** (la mosca no tiene ratón) y queda documentado.
  - Las teclas salen de un alfabeto de acciones pequeño, no de texto libre.
- **Notas en pantalla:** las escribe RWKV-7 a partir del estado del bus (qué percibe, qué quiere hacer, con qué probabilidad según Laya). Aparecen en la capa fantasma.

## 6. Escalera de autonomía (modo fantasma)

| Nivel | Qué pasa | Cómo se sube |
|---|---|---|
| **0 · Fantasma** | Clics, teclas y notas se **dibujan** semitransparentes; nada se ejecuta. Se registra todo | — |
| **1 · Validado** | Laya evalúa cada acción ("¿es válida?", p) y se compara con lo que tú harías o con el objetivo de la tarea | Acierto ≥ umbral durante N acciones (p. ej., ≥ 90 % en 500) |
| **2 · Autónomo por tarea** | Las acciones con p ≥ umbral se ejecutan de verdad en esa tarea o app | **Métrica + tu permiso explícito** |
| **3 · Autónomo general** | Trabaja solo en el sandbox; tú supervisas | Métrica sostenida + tu permiso. Se revoca con un botón |

El sistema **pide** permiso para subir de nivel; nunca se lo concede a sí mismo.

## 7. Tests del sandbox (de reflejo a trabajo)

1. Seguir con el cursor un objeto que se mueve (reflejo optomotor).
2. Huir de o evitar un estímulo "amenaza" (looming).
3. Clicar la "comida" y no el "veneno" (discriminación con recompensa).
4. Pulsar la tecla correcta ante un símbolo (asociación).
5. AND / OR / **XOR** y retardo con memoria.
6. Tareas de escritorio simples: abrir una app, cerrar una ventana, escribir un texto dictado.
7. Trabajo autónomo en una mini-tarea definida.

Cada test se corre también con el **conectoma barajado** y con **Laya/RWKV apagados**. Expectativa realista: los reflejos (1–3) son probables; lo demás depende de cuánto aporten Laya, la memoria y el aprendizaje, y medir eso es el experimento.

## 8. Aprendizaje: MEM-EGGROLL (memoria que guía la evolución)

Objetivo: aprender **en la nube sin GPU**, sin backprop por la red spiking y sin tocar la anatomía.

**Piezas que existen (verificadas):**

| Pieza | Qué aporta | Código |
|---|---|---|
| **EGGROLL** (ES low-rank, `ΔW = ABᵀ`) | Evolución con poblaciones grandes y solo pasadas hacia delante; demostrado en RWKV-7 | `external/learning/HyperscaleES`, `nano-egg` |
| **SEAL** | El modelo genera sus propias "auto-ediciones" (datos o directivas de ajuste) y aprende a generarlas con RL | `external/learning/SEAL` |
| **IER / SER** (ESER/XSER/MSER) | Repetición inmediata y espaciada de episodios exitosos | `external/learning/Repetition` (MIT) |
| **Reincarnating RL** | Reutilizar políticas y cómputo previos | `external/learning/reincarnating_rl` |
| **HeLa-Mem** | Grafo episódico hebbiano + consolidación semántica | `external/memory/HeLa-Mem` |
| **SYNAPSE** | Activación propagada + decaimiento + inhibición lateral | Paper; el código "se publicará" (no disponible) |
| **CMA-ES** | ES clásico para genomas pequeños | `external/learning/pycma` |

**Nuestra fusión (hipótesis nueva, a validar):**

1. **Episodios estructurados:** contexto, eventos (VER, MOVER, CLIC…) y resultado (recompensa, novedad, sorpresa), guardados en un **grafo episódico → semántico → procedural**.
2. **Recuperación por activación propagada** con decaimiento e inhibición lateral: el estado actual activa recuerdos y conceptos relacionados.
3. **El "reactor de memoria" decide:** repetir (IER/SER), revivir un fallo cambiando solo el tramo crítico, o explorar (EGGROLL).
4. **EGGROLL guiado por memoria:**
   - `Δθ = M_memoria ⊙ (ABᵀ)`: la memoria enfoca qué subespacio mutar;
   - el **rango** `r` crece con la incertidumbre, la novedad y el conflicto entre recuerdos.
5. **SEAL sin gradientes:** RWKV genera sus auto-ediciones (notas, reglas, ejemplos) y la actualización de pesos se hace con **EGGROLL low-rank** en vez de SFT. Esto lo hace viable en CPU y es parte de la novedad.
6. **Reencarnación:** cada generación de política renace con sus pesos **y** su memoria.
7. **Vigilia / sueño:** de día actúa; de "noche" repite, consolida, recombina trayectorias (A→B + B→C ⇒ A→C, contrafactuales) y olvida.

**Qué se evoluciona:**
- en el cerebro: ganancias por tipo celular y la regla de plasticidad (nunca la conectividad);
- en Laya: temperatura y política de validación;
- en RWKV-7: adaptadores low-rank.

**Criterio de éxito:** aprender más rápido y mejor que (a) ES sin memoria, (b) el conectoma barajado y (c) sin repetición. Si no mejora, se reporta.

## 9. Dónde corre

| Entorno | Qué | Estado |
|---|---|---|
| **Nube (este contenedor)** | 4 CPU, 15 GB RAM, sin GPU, Docker, efímero. Cerebro, neko, servicios Rust y MEM-EGGROLL en CPU | Primero |
| **VM (QEMU/VirtualBox)** | ISO de BrainOS (Arch + Okimarchy/Niri + servicios + neko) | Segundo |
| **USB en el portátil** (ROG Strix G18, 8 GB VRAM) | BrainOS arrancable. Aquí el cerebro puede usar GPU | Último. El portátil no debe sufrir: solo cuando esté listo |

Persistencia: todo lo útil se commitea. Pesos y datos en `vault/` (trozos <95 MB + sha256); lo gigante, por `tools/vault.py fetch --all`.

## 10. Fases e hitos

| Fase | Entregable | Hecho cuando |
|---|---|---|
| **0 ✅ Base** | Shiu LIF en CPU (azúcar → MN9: 83 Hz v630, 79 Hz v783) | Hecho |
| **1 · Controles** | Conectoma barajado + 30 ensayos + curva de frecuencia | Diferencia real vs. barajado cuantificada |
| **2 · Sustrato** | Formato común + port a MaleCNS; BANC como segundo animal | Mismos tests en los tres |
| **3 · Esqueleto BrainOS** | Workspace Rust (`brainosd`, `bus`) + neko corriendo en la nube | neko accesible y el bus transmite estado |
| **4 · Retina + motor** | Píxeles → fotorreceptores; descendentes → ratón (en fantasma) | Test 1 (seguir objeto) medido vs. barajado |
| **5 · Dashboard 3D** | Cerebro en directo junto al escritorio | Ves spikes y overlay fantasma a la vez |
| **6 · Laya córtex** | Laya en Rust: escucha (legibilidad) y valida acciones | Tabla vs. azar, barajado y lineal |
| **7 · Voz** | RWKV-7 en Rust escribe notas del estado en el overlay | Notas coherentes con el estado (evaluado) |
| **8 · MEM-EGGROLL** | Memoria en grafo + EGGROLL guiado + SEAL sin gradientes + sueño | Curvas vs. ES sin memoria y vs. barajado |
| **9 · Autonomía** | Escalera fantasma → validado → autónomo con permiso | Nivel 2 alcanzado en alguna tarea |
| **10 · Marcadores** | PCI (PCIst/LZ), persistencia, PyPhi IIT 4.0 en microcircuitos, comparación con TVB humano | Informe con resultados y límites |
| **11 · S.O. arrancable** | ISO Arch + Okimarchy/Niri + BrainOS → VM → USB | Arranca en VM y en el portátil |

## 11. Riesgos y límites honestos

- **LIF es una simplificación**; las sinapsis son predichas (BANC F1 ≈ 0,83) y el signo sale del neurotransmisor.
- **El mapeo píxel→fotorreceptor y neurona→ratón es inventado por necesidad:** se marca como hipótesis y se contrasta con el conectoma barajado.
- **Un cerebro de mosca no va a "usar un ordenador" como un humano.** La autonomía real dependerá de Laya, la memoria y el aprendizaje, y medir **cuánto aporta cada parte** es el resultado.
- **Velocidad:** ~8 s de CPU por segundo biológico del cerebro completo. El tiempo real en la nube exige subcircuitos o GPU (portátil o alquilada).
- **SEAL y EGGROLL están probados en LLM, no con un cerebro spiking;** la fusión MEM-EGGROLL es nueva y puede fallar.
- **Licencias:** varios submódulos son GPL o no tienen licencia; no se copia su código al userland propio.
- **Nada de esto demuestra conciencia.** Se reportan marcadores con controles.

## 12. Fuera de alcance (por ahora)

Cuerpo biomecánico (FlyGym), biofísica multicompartimental masiva, kernel propio en Rust, acciones fuera del sandbox, acceso de red libre del cerebro.

## 13. Decisiones pendientes

- Licencia del código propio (MIT/Apache).
- Neuronas concretas de entrada (fotorreceptores) y salida (descendentes) y el decodificador de acciones.
- Umbrales exactos de la escalera de autonomía.
- Si se alquila una GPU puntual para las fases 8–10.

## 14. Fuentes principales

BANC https://github.com/htem/BANC-project · MaleCNS https://male-cns.janelia.org/ · FlyWire v783 https://zenodo.org/records/10676866 · Shiu LIF https://github.com/philshiu/Drosophila_brain_model · Laya https://huggingface.co/convaiinnovations/laya · RWKV https://github.com/BlinkDL/RWKV-LM · candle-rwkv https://github.com/nkypy/candle-rwkv · web-rwkv https://github.com/cryscan/web-rwkv · neko https://github.com/m1k1o/neko · Niri https://github.com/YaLTeR/niri · Okimarchy https://github.com/cristian-fleischer/okimarchy · Omarchy https://github.com/basecamp/omarchy · EGGROLL https://arxiv.org/abs/2511.16652 · https://github.com/ESHyperscale/HyperscaleES · SEAL https://arxiv.org/abs/2506.10943 · https://github.com/Continual-Intelligence/SEAL · IER/SER https://github.com/UoA-CARES/Repetition · Reincarnating RL https://arxiv.org/abs/2206.01626 · HeLa-Mem https://arxiv.org/abs/2604.16839 · SYNAPSE https://arxiv.org/abs/2601.02744 · Catálogo completo: `docs/research/catalogo_recursos.md`
