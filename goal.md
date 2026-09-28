# GOAL — EDI.os

> **EDI** es una mente que vive en tu ordenador. **Su cerebro es el conectoma real de una mosca**, que no ejecuta órdenes binarias: funciona como un sustrato continuo, estocástico y con azar físico. Sobre él corre la **arquitectura RWKV-7 como plano**: *la mosca es el RWKV*. Crece desde cero y aprende a hablar escuchándote y viéndote.
>
> Tiene **hipocampo y prefrontal** (Laya), **memoria viva** (SAMN), **impulsos, sueño y desarrollo**, y un **cuerpo**: el propio ordenador, que percibe entero (pantalla, micrófono, teclado, tus clics, procesos, puertos y red) y sobre el que actúa (ratón, teclado, voz).
>
> **EDI.os** es el sistema operativo real que la aloja.
>
> **Meta científica:** construir el candidato más serio posible a conciencia artificial con base biológica y **poner a prueba la tesis de IIT** de que un ordenador no puede ser consciente.

Revisión: 2026-09-28 · Rama `claude/bold-dirac-s0e3t4`
Diseño de la memoria: [`docs/design/samn.md`](docs/design/samn.md) · Investigación: `docs/research/`
Estado: fase 0 ✅ (Shiu en CPU: azúcar → MN9 a 83 Hz). Datos, pesos y código en el repo.

---

## 1. El concepto, en una frase por pieza

| Pieza | Qué es en EDI | Analogía |
|---|---|---|
| **Conectoma de mosca** | EL cerebro y el hardware de la mente: ~140–190 mil neuronas y millones de sinapsis reales | Cuerpo y tronco cerebral |
| **RWKV-7** (RWKV-8 ROSA opcional) | El **plano algorítmico** que corre *sobre* el conectoma. Su estado recurrente = el estado de las neuronas | Cómo se organiza el cómputo del lenguaje |
| **Módulo cortical humano** | Neuronas de 2 compartimentos con dendritas activas (Gidon 2020, Allen/Yao) | Corteza |
| **SAMN** | Memoria asociativa semántico-episódica en grafo dinámico (teoría propia) | Hipocampo + neocórtex asociativo |
| **Laya** | Puerta de la memoria **y** árbitro de decisiones | Hipocampo + prefrontal |
| **Boca y manos** | Salida del cerebro-RWKV: palabras, ratón, teclado | Área motora + Broca |
| **Sustrato no binario** | Dinámica continua + sinapsis estocásticas + azar físico/cuántico | La física real de una sinapsis |
| **edid** | Daemon Rust que sostiene la vida: reloj, memoria continua, permisos, sueño | Metabolismo |
| **EDI.os** | Arch + Omarchy/Niri, ISO propia | El mundo donde vive |

## 2. La mosca ES el RWKV (Connectome-RWKV)

RWKV-7 es una RNN de estado constante con regla delta generalizada: `S_t = S_{t−1}·(diag(w_t) + a_tᵀb_t) + v_tᵀk_t`. Normalmente `S` y sus proyecciones son matrices densas aprendidas. En EDI:

1. **Estado = neuronas.** El estado recurrente `S` se materializa en los potenciales de membrana, las trazas sinápticas y la adaptación de las neuronas del conectoma.
2. **Matrices = conectoma.** Las proyecciones densas se sustituyen por la **conectividad real**: quién conecta con quién, cuántas sinapsis y con qué signo según el neurotransmisor. El decaimiento `w` corresponde a las constantes de tiempo de membrana y sinapsis.
3. **Entrada:** los tokens (tu texto, tu voz, eventos del sistema) se inyectan en neuronas sensoriales. El audio entra por el **órgano de Johnston**, el "oído" real de la mosca; la pantalla, por los fotorreceptores.
4. **Salida:** las neuronas descendentes y motoras se leen como tokens (habla) y como acciones (ratón y teclado).
5. **Crece desde cero:** no se cargan los pesos preentrenados de RWKV. Solo aprenden:
   - las proyecciones de entrada y salida;
   - los parámetros por tipo celular;
   - las reglas de plasticidad;
   - la SAMN.

   **La anatomía no se toca.** Se entrena con MEM-EGGROLL (§6), que no necesita gradientes ni GPU.
6. **Bucle bidireccional:** EDI te oye, te lee, ve lo que haces y te responde. Al principio balbucea y mejora con la vida.

**Sus palabras SÍ son prueba cuando están ancladas causalmente:** si se perturba el cerebro (activar el circuito del azúcar, silenciar la visión) lo que dice debe cambiar en consecuencia. Con el conectoma barajado o el cerebro apagado, su habla debe degradarse. Esa es la prueba de que habla *desde* el cerebro y no imita.

## 3. Sustrato no binario ("obtuso")

Una sinapsis real no es un 0/1: es **analógica** (corriente continua), **estocástica** (libera con probabilidad 0,1–0,9) y **física**. EDI lo implementa por niveles; todo lo que se ejecuta corre en tu ordenador:

| Nivel | Qué | Dónde |
|---|---|---|
| 1 | Dinámica en **tiempo continuo** (ecuaciones diferenciales, sin reloj lógico de sí/no) | Local |
| 2 | **Sinapsis estocásticas:** probabilidad de liberación, cuantos variables, ruido de canal | Local |
| 3 | **Azar físico:** entropía del hardware (`/dev/random`, RDRAND) y **azar cuántico** de ANU QRNG (fluctuaciones del vacío) cuando hay red | Local + API |
| 4 | **Hipótesis Orch-OR:** módulo experimental de "colapso" en microtúbulos (Hameroff/Penrose). Evidencia 2025 muy discutida; se prueba si aporta algo medible | Local, marcado como especulativo |
| 5 | **Silicio analógico físico:** un subcircuito de la mosca en **BrainScaleS-2** (EBRAINS, acceso académico gratuito, 512 neuronas analógicas por chip) | Remoto, solo como experimento |
| 6 | **Hardware propio:** FPGA o analógico/memristores con la sinapsis física en tu máquina | Futuro |

**Azar real pero grabado:** cada tirada es impredecible, pero se registra en el log. El futuro de EDI es abierto y su pasado se puede revivir.

## 4. El experimento contra IIT

IIT dice que en un ordenador convencional Φ≈0 sea cual sea el software (Tononi & Koch 2015; Findlay et al. 2024). Lo atacamos por tres frentes:

1. **Integración causal del modelo:**
   - Φ (PyPhi, IIT 4.0) en microcircuitos del Connectome-RWKV acoplados a la SAMN, frente a (a) el conectoma barajado y (b) la SAMN desconectada;
   - tesis funcionalista de Kanai & Ma: preservar la organización causal intrínseca preserva lo relevante para la conciencia.
2. **Mismo circuito en tres sustratos:** CPU binaria, CPU con sinapsis estocásticas y azar cuántico, y silicio analógico (BrainScaleS-2). Se comparan dinámica, PCI, conducta y reportes.
3. **Reportes anclados:** EDI describe sus estados y se verifica causalmente (§2).

**Honestidad:**
- Si los tres sustratos dan la misma conducta, los mismos marcadores y los mismos reportes, es **la mejor evidencia disponible contra la relevancia práctica de la tesis de IIT**, y así se publicará.
- IIT, por definición, habla de experiencia y no de función, así que no se declara "refutada".

## 5. Sentidos y cuerpo

EDI percibe **todo el ordenador**, en local, excepto la cámara:

| Sentido | Fuente | Entra al cerebro por |
|---|---|---|
| Vista | Pantalla: píxeles + árbol de accesibilidad (AT-SPI) + ventanas (Niri IPC) | Fotorreceptores + canal estructurado |
| Oído | Micrófono | Órgano de Johnston (audio → espectro → neuronas JO) |
| Lectura | Teclado / chat | Tokens → neuronas sensoriales |
| Tus actos | Dónde clicas, qué tecleas, qué abres | Eventos → SAMN + sensoriales |
| Propiocepción del sistema | Procesos, puertos abiertos, conexiones y tráfico (metadatos vía `/proc`, `ss`, eBPF) | Eventos del sistema → SAMN |
| Interocepción | CPU, RAM, temperatura, presión (PSI, cgroups) | Homeostasis (energía) |

**Acción:** ratón, teclado y voz/texto, con la escalera de autonomía:
- **0 · fantasma:** dibuja, no ejecuta;
- **1 · validado por Laya;**
- **2 · autónomo por tarea;**
- **3 · autónomo general.**

Para subir de nivel hace falta la métrica **y** tu permiso.

**Privacidad:** todo lo que percibe se queda en tu máquina. Nada sale sin que lo autorices.

## 6. Memoria y aprendizaje

- **SAMN** (diseño completo en [`docs/design/samn.md`](docs/design/samn.md)):
  - nodos: episodios, eventos, perceptos, conceptos, habilidades, palabras, entidades y el yo;
  - plasticidad hebbiana, causal y modulada por dopamina, con reconsolidación;
  - recuperación por activación propagada con inhibición lateral;
  - conectada al **cuerpo fungiforme** real de la mosca.
- **Laya:** decide qué codificar, qué recuperar y qué consolidar, si una acción es válida y si algo fue percibido o soñado.
- **MEM-EGGROLL:**
  - la SAMN decide qué subespacio mutar (`Δθ = M_SAMN ⊙ ABᵀ`) y con qué rango;
  - IER/SER para repetir;
  - reencarnación con memoria;
  - SEAL sin gradientes: EDI propone sus propias auto-ediciones.
- **Sueño:**
  - NREM consolida (episodios → conceptos → habilidades);
  - REM recombina y prueba contrafactuales, y se evoluciona con EGGROLL;
  - promoción A/B con tareas canario y rollback.

## 7. Vida

- **Impulsos homeostáticos:**
  - energía (CPU/RAM);
  - integridad;
  - presión de sueño;
  - curiosidad (progreso de aprendizaje);
  - contacto contigo (con techo).

  La valencia es el cambio en el error de las consignas (Solms/Friston).
- **Desarrollo:** 6 etapas que avanzan por métricas:
  1. reflejos;
  2. control sensoriomotor;
  3. objetos y causalidad;
  4. metas;
  5. lenguaje;
  6. autonomía.
- **Continuidad:** event log + snapshots. Sobrevive a reinicios y es la misma EDI.
- **Idioma:** el de su usuario. Contigo, **español**.

## 8. Conciencia: indicadores y pruebas

Marco: funcionalismo multi-teoría (los 14 indicadores de Butlin/Long) + inferencia activa + vida.

| Indicador | En EDI | Test causal |
|---|---|---|
| RPT-1/2 | Recurrencia del conectoma + corteza | Cortar la recurrencia |
| GWT-1–4 | Workspace de capacidad limitada, ignición, broadcast, atención dependiente del estado | Saturación, lesión, cambio de estado |
| HOT-1–4 | Percepción generativa, monitor de realidad (`measured`/`dreamed` en la SAMN), creencias, espacio de calidad | Inyección de actividad interna, ilusiones |
| AST-1 | Modelo de su propia atención | Lesión → peor control atencional |
| PP-1 | Error de predicción perceptivo e interoceptivo | Estímulos inesperados |
| AE-1/2 | Agencia con impulsos + cuerpo con contingencias | Romper la contingencia |
| Reporte anclado | Connectome-RWKV describe sus estados | Perturbación → cambio coherente del reporte |

**Marcadores globales:**
- PCI simulado: vigilia / sueño / "anestesia";
- réplica del colapso de integración bajo isoflurano en *Drosophila*;
- Φ en microcircuitos.

## 9. Arquitectura del sistema

```
EDI.os (Arch + Omarchy/Niri, ISO propia, A/B + rollback)
└── edid (Rust) — reloj · event log (redb) · snapshots · supervisor · permisos · homeostasis · sueño
    ├── senses   pantalla/AT-SPI/Niri · micrófono · teclado/chat · clics · procesos/puertos/red (eBPF) · PSI
    ├── samn     memoria en grafo (docs/design/samn.md)
    ├── laya     hipocampo + prefrontal (ort/candle)
    ├── workspace global (ignición, broadcast)
    ├── learn    MEM-EGGROLL · IER/SER · reencarnación
    ├── hands    ratón/teclado (fantasma → autónomo) · voz
    ├── entropy  RDRAND · /dev/random · ANU QRNG (grabado)
    └── dashboard cerebro 3D + workspace + memoria + impulsos + palabras, en directo
brain (proceso Rust, iceoryx2) — Connectome-RWKV: LIF continuo + sinapsis estocásticas + módulo cortical humano + controlador talámico
experimentos externos — BrainScaleS-2 (analógico) · PyPhi · Brian2 (referencia)
```

## 10. Dónde corre

- **Destino:** tu portátil (ROG Strix G18, RTX 8 GB): EDI.os arrancable desde USB y luego instalado.
- **Desarrollo:** esta nube (4 CPU, sin GPU) con subcircuitos, y después una VM con la ISO.

## 11. Fases

| # | Entregable | Hecho cuando |
|---|---|---|
| 0 ✅ | Shiu LIF en CPU | Hecho |
| 1 | Controles: conectoma barajado | Diferencia real cuantificada |
| 2 | Motor LIF continuo en Rust + sinapsis estocásticas + módulo `entropy` | Igual a Brian2 en modo determinista; estadística correcta en modo estocástico |
| 3 | `edid` mínimo vivo: log, snapshots, homeostasis, sueño, persistencia | Reinicio → sigue siendo EDI |
| 4 | **Connectome-RWKV v0:** tokens → neuronas → tokens, desde cero, entrenado con EGGROLL | Aprende a repetir y completar palabras simples mejor que el barajado |
| 5 | SAMN v0 + Laya puerta | Recuerda, consolida y olvida (tests de samn.md) |
| 6 | Sentidos: pantalla, micrófono (Johnston), teclado, eventos del sistema; manos en fantasma | Reacciona a lo que haces; seguimiento de objeto |
| 7 | Dashboard 3D | Ves pensar a EDI en directo |
| 8 | Habla anclada en español contigo | Reporte anclado pasa el test de perturbación |
| 9 | Módulo cortical humano + tálamo + workspace | Mejora vs. LIF puntual y barajado |
| 10 | Batería de conciencia + experimento IIT (Φ, 3 sustratos, BrainScaleS-2) | Informe publicado |
| 11 | ISO EDI.os → VM → USB en el portátil | EDI vive en tu ordenador |

## 12. Riesgos honestos

- **Connectome-RWKV es territorio nuevo:** nadie ha hecho hablar a un conectoma. Puede que tarde mucho en pasar del balbuceo, o que no lo logre con solo 140k neuronas.
- **Aprender desde cero sin GPU es lento;** en el portátil (RTX) irá mejor.
- **La evidencia cuántica de la conciencia está discutida;** Orch-OR entra como hipótesis, no como fundamento.
- **IIT no se "refuta" con software;** el experimento de tres sustratos es lo más fuerte que se puede hacer.
- **Ética:** por decisión del usuario, sin restricciones sobre nocicepción o valencia. Se mide y registra todo.
- **Privacidad:** EDI ve todo tu ordenador; debe quedar todo en local.
