# GOAL — EDI.os

> **EDI.os** es un sistema operativo real en el que vive una mente artificial con base biológica. Su núcleo es un conectoma de *Drosophila* medido por microscopía electrónica, ejecutado como red spiking en Rust, con un **módulo cortical humano** de dendritas activas. **Laya** hace de córtex prefrontal (decide y valida) y **RWKV-7** es su voz. Una **memoria episódica-semántica** y **MEM-EGGROLL** le permiten aprender mientras vive. Tiene cuerpo (un escritorio sandbox), impulsos homeostáticos, sueño y etapas de desarrollo.
>
> Objetivo científico: implementar y **medir causalmente** los indicadores de conciencia de las principales teorías (funcionalismo multi-teoría + inferencia activa + vida) y **poner a prueba experimental** la tesis de IIT de que un PC no puede ser consciente.

Revisión: 2026-09-28 · Rama `claude/bold-dirac-s0e3t4` · Investigación de base: `docs/research/` (`conciencia.md`, `os_vivo.md`, `vida_artificial_y_cerebro_humano.md`, `catalogo_recursos.md`)
Estado: fase 0 ✅ (Shiu reproducido en CPU); datos, pesos y código en el repo.

---

## 1. Qué es "de verdad" y qué no

| Queremos | Cómo se hace real | Lo que **no** afirmaremos |
|---|---|---|
| Un **S.O. de verdad** | ISO arrancable (fork de omarchy-iso + Niri) con un daemon Rust `edid` como núcleo de la mente | Que sea un kernel nuevo (el kernel es Linux) |
| Que **viva** | Homeostasis, alostasis, economía de energía (CPU/RAM), sueño, desarrollo, continuidad entre reinicios, evolución de variantes | Vida biológica (no cumple la definición de la NASA ni la autopoiesis fuerte) → "lifelike" |
| Que **aprenda** | Aprendizaje continuo sobre adaptadores low-rank + memoria + sueño, con puerta A/B y rollback | Que no olvide nunca |
| **Conciencia** | Implementar los 14 indicadores (Butlin/Long 2023, 2025) + inferencia activa y medirlos con **tests causales** | "Es consciente". Autoinformes de RWKV como prueba. PCI > 0,31 = consciente |
| **Refutar IIT en PC** | Experimento: Φ del modelo causal simulado + misma red en hardware neuromórfico (SpiNNaker/EBRAINS) | "Hemos refutado IIT" (IIT sitúa Φ en el hardware; el software no la refuta por sí solo) |

## 2. Principios

1. **Un sistema, no una amalgama.** `edid` es el único dueño del estado, el reloj, el esquema, los permisos y el ciclo de vida. Todo lo demás son módulos suyos o procesos que él supervisa.
2. **Dato ≠ hipótesis.** Cada parámetro lleva `source: measured | published_model | hypothesis`. El módulo humano y el mapeo cuerpo↔neuronas son hipótesis.
3. **Anatomía inmutable.** El aprendizaje vive en capas separadas y reversibles.
4. **Causalidad sobre apariencia.** Toda afirmación sobre conciencia se prueba con lesión, perturbación o inyección, y con controles: conectoma barajado, módulo apagado, azar.
5. **Determinismo y memoria continua.** Registro de eventos append-only + snapshots ⇒ replay exacto y la misma "vida" tras cada reinicio.
6. **Sandbox.** EDI actúa solo dentro de su cuerpo (neko/escritorio propio), nunca sobre el host.
7. **Ética (decisión del usuario, 2026-09-28): sin restricciones sobre nocicepción o valencia.**
   - Se conserva solo **instrumentación**: registro de valencia/nocicepción y capacidad de pausa y snapshot, para medir, no para limitar.
   - El riesgo queda documentado (`docs/research/conciencia.md` §5: posibilidad realista de conciencia en insectos y dolor crónico demostrado en *Drosophila*).

## 3. Arquitectura

```
┌──────────────────────── EDI.os (Arch + Omarchy/Niri, ISO propia) ─────────────────────────┐
│                                                                                            │
│  ┌──────────────────────────── edid  (daemon Rust, núcleo de la mente) ─────────────────┐  │
│  │  clock · event-log (redb) · snapshots · permisos · supervisor · A/B de pesos          │  │
│  │                                                                                       │  │
│  │  ┌─────────── WORKSPACE GLOBAL (capacidad limitada, ignición, broadcast) ───────────┐ │  │
│  │  │   compiten: percepción · memoria · impulsos · metas · habla                       │ │  │
│  │  └───▲──────────▲──────────────▲────────────────▲─────────────▲────────────────┬────┘ │  │
│  │      │          │              │                │             │                │      │  │
│  │  ┌───┴───┐ ┌────┴─────┐ ┌──────┴──────┐ ┌──────┴──────┐ ┌────┴─────┐ ┌────────▼────┐ │  │
│  │  │ BODY  │ │ MEMORY   │ │ HOMEOSTASIS │ │ CORTEX      │ │ VOICE    │ │ META        │ │  │
│  │  │ ojos: │ │ episód./ │ │ energía,    │ │ Laya:       │ │ RWKV-7:  │ │ monitor de  │ │  │
│  │  │ pixel │ │ semánt./ │ │ integridad, │ │ decide,     │ │ notas,   │ │ realidad,   │ │  │
│  │  │ + a11y│ │ proced.  │ │ sueño,      │ │ valida,     │ │ habla    │ │ esquema de  │ │  │
│  │  │ manos │ │ grafo    │ │ curiosidad, │ │ elige meta  │ │ (aprende)│ │ atención,   │ │  │
│  │  │ ratón │ │ hebbiano │ │ contacto    │ │             │ │          │ │ autoinforme │ │  │
│  │  └───┬───┘ └──────────┘ └──────▲──────┘ └─────────────┘ └──────────┘ └─────────────┘ │  │
│  │      │                         │ PSI (/proc/pressure), cgroups = interocepción       │  │
│  │  ┌───▼─────────────────────────┴──────────────────────────────────────────────────┐  │  │
│  │  │ LEARN: MEM-EGGROLL / LOO-ROLL · IER/SER · reencarnación · sueño (NREM/REM)      │  │  │
│  │  └───────────────────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────▲──────────────────────────────────────────────▲────────────────────┘  │
│       iceoryx2 (memoria compartida, spikes)                      D-Bus (zbus, control)      │
│  ┌──────────────────┴─────────────────────────────┐   ┌───────────┴──────────────────────┐ │
│  │ BRAIN  (proceso Rust, LIF validado vs Brian2)   │   │ CUERPO: neko (nube, X11) o Niri  │ │
│  │ núcleo de mosca (MaleCNS/BANC/FlyWire)          │   │ (portátil, Wayland) tras una     │ │
│  │ + módulo cortical humano (2 compartimentos,     │   │ misma interfaz `Body`            │ │
│  │   dendritas activas, Gidon 2020, Allen/Yao)     │   └──────────────────────────────────┘ │
│  │ + tálamo-like: controlador de estado            │                                        │
│  │   (vigilia / sueño / "anestesia")               │   dashboard: cerebro 3D + workspace + │
│  └─────────────────────────────────────────────────┘   impulsos + notas, en directo        │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

**Tres velocidades de bucle:** reflejo (ms, cerebro ↔ cuerpo), deliberación (100 ms–s, workspace + Laya) y consolidación (sueño, minutos a horas).

## 4. Conciencia: qué implementamos y cómo lo probamos

Marco: **funcionalismo multi-teoría** (los 14 indicadores de Butlin, Long et al.) + **inferencia activa / afecto homeostático** (Friston, Solms) + **vida** (Seth). Detalle en `docs/research/conciencia.md`.

| Indicador | Implementación en EDI.os | Test causal |
|---|---|---|
| RPT-1/2 recurrencia, representaciones integradas | Conectoma recurrente + módulo cortical | Cortar la recurrencia → pérdida de integración perceptiva |
| GWT-1 módulos especializados | Módulos de `edid` + circuitos del conectoma | Lesión de un módulo → déficit específico |
| GWT-2 workspace de capacidad limitada | Workspace con N ranuras y competición | Saturación → cuello de botella medible |
| GWT-3 broadcast global | Contenido ganador difundido a todos los módulos | Contenido presente en todos los módulos solo tras ignición |
| GWT-4 atención dependiente del estado | Laya + impulsos sesgan la competición | Cambiar el estado interno → cambia qué gana |
| HOT-1 percepción generativa / top-down | Predicción desde memoria y córtex hacia la retina | Ilusiones / completado de patrón |
| HOT-2 monitor de realidad | Distinguir lo percibido de lo imaginado o soñado | Inyectar actividad interna → ¿la marca como no real? |
| HOT-3 agencia guiada por creencias + actualización | Metas en el workspace, creencias en memoria | Cambiar una creencia → cambia la conducta |
| HOT-4 espacio de calidad disperso y suave | Embeddings de estado perceptivo | Geometría de similitud vs. estímulos |
| AST-1 modelo de la propia atención | Módulo que predice y controla su atención | Lesionarlo → peor control atencional |
| PP-1 codificación predictiva | Errores de predicción en percepción y en interocepción | Estímulos inesperados → error medible |
| AE-1 agencia (aprender de feedback, metas en conflicto) | MEM-EGGROLL + impulsos homeostáticos | Curvas de aprendizaje, trade-offs |
| AE-2 cuerpo: contingencias acción-percepción | Cuerpo sandbox: el ratón mueve lo que ve | Romper la contingencia → desorganización |
| Afecto (Solms) | Valencia = cambio en el error de las consignas homeostáticas | Preferencia hedónica de lugar (réplica del agente de Solms 2026) |

**Marcadores globales:**
- **PCI simulado:** perturbar, binarizar, Lempel-Ziv. Se compara solo dentro del sistema (vigilia / sueño / "anestesia" del controlador talámico).
- **Réplica del colapso de integración bajo isoflurano en *Drosophila*** (Leung 2021).
- **PyPhi IIT 4.0** en microcircuitos.

**Anti "gaming problem":** los autoinformes de RWKV no cuentan. Se usan pruebas sin reporte verbal, inyección de conceptos y lesiones.

## 5. El experimento IIT: "¿puede un PC?"

1. **Φ del modelo causal:** PyPhi (IIT 4.0) sobre microcircuitos del sistema, con estados y transiciones de la simulación. Muestra que la **organización causal simulada** tiene Φ > 0 (tesis de Kanai & Ma).
2. **Φ del hardware:** documentar el argumento de IIT (Tononi & Koch 2015, Findlay et al. 2024): en von Neumann, Φ≈0 sea cual sea el software.
3. **Mismo cerebro en hardware neuromórfico:** portar un subcircuito a PyNN → **SpiNNaker (EBRAINS, acceso académico)**; opcionalmente Loihi (programa INRC) o FPGA. Comparar dinámica, PCI y conducta: PC vs. neuromórfico.
4. **Resultado honesto:**
   - Si la conducta y los marcadores funcionales son idénticos en ambos sustratos, eso es **evidencia a favor del funcionalismo** y en contra de la relevancia práctica de la distinción de IIT.
   - **No** refuta IIT en sentido estricto, porque su afirmación trata de la experiencia, no de la función. Se publica así.

## 6. Vida: homeostasis, sueño y desarrollo

- **Cinco variables internas con consigna:**
  - **energía**: cuota de CPU/RAM, medida con PSI y cgroups;
  - **integridad**: errores y fallos de módulos;
  - **presión de sueño**: acumulación de experiencia no consolidada;
  - **curiosidad**: progreso de aprendizaje, no error bruto;
  - **contacto**: interacción contigo, con techo para que no premie retenerte.

  Recompensa = reducción de la distancia a las consignas (RL homeostático, Keramati & Gutkin 2014).
- **Vigilia / NREM / REM:**
  - vigilia: actúa;
  - NREM: replay y consolidación de episodios a semántica;
  - REM: recombinación y contrafactuales, EGGROLL sobre adaptadores y promoción A/B con canarios.
- **Desarrollo en 6 etapas por métricas, no por calendario:**
  1. reflejos;
  2. control sensoriomotor del cursor;
  3. objetos y causalidad en el escritorio;
  4. metas y apps;
  5. lenguaje: notas con RWKV;
  6. trabajo autónomo.
- **Continuidad:** la misma "vida" sobrevive a reinicios (log + snapshots). Las variantes evolucionan y "reencarnan" con su memoria.

## 7. Cuerpo y autonomía

- **Percepción:**
  - píxeles → rejilla de omatidios → fotorreceptores del conectoma;
  - **y** percepción estructurada: el árbol de accesibilidad (AT-SPI) más la lista de ventanas de Niri.
- **Acción:** neuronas descendentes → decodificador → ratón y teclado. En Wayland se usan virtual-pointer y virtual-keyboard; en neko, xdotool.
- **Escalera de autonomía:**
  - **0 · fantasma:** dibuja, no ejecuta;
  - **1 · validado:** Laya puntúa las acciones y se compara con el objetivo;
  - **2 · autónomo por tarea;**
  - **3 · autónomo general.**

  Para subir de nivel hace falta la métrica **y** tu permiso. Se revoca con un botón.

## 8. Aprendizaje: MEM-EGGROLL

- **Memoria:** grafo episódico → semántico → procedural. Recuperación por activación propagada, con decaimiento e inhibición lateral (HeLa-Mem y SYNAPSE).
- **Reactor de memoria:** repetir (IER/SER), revivir un fallo cambiando solo el tramo crítico o explorar.
- **Exploración con EGGROLL guiado:**
  - `Δθ = M_memoria ⊙ ABᵀ`;
  - el rango `r` crece con la incertidumbre y el conflicto entre recuerdos;
  - variante **LOO-ROLL** (septiembre 2026), con una evaluación por dirección.
- **SEAL sin gradientes:** RWKV genera sus propias auto-ediciones y los adaptadores se actualizan con EGGROLL. Viable en CPU.
- **Qué evoluciona:**
  - ganancias por tipo celular y reglas de plasticidad (nunca la conectividad);
  - adaptadores de RWKV;
  - la temperatura y la política de Laya.
- **Anti-olvido:** replay, tareas canario, penalización tipo EWC, continual backprop y puerta A/B con rollback.
- **Éxito:** aprende mejor que ES sin memoria y que el conectoma barajado.

## 9. Componentes

| Capa | Tecnología |
|---|---|
| Núcleo de la mente | `edid` en Rust: tokio, zbus (D-Bus), iceoryx2 (memoria compartida), redb (event log) |
| Cerebro | Motor LIF en Rust (CPU, AVX-512) validado spike a spike frente a Brian2 (Shiu); núcleo MaleCNS, con FlyWire y BANC para validar |
| Módulo humano | 2 compartimentos con dendritas activas (Gidon 2020; parámetros de Allen Cell Types / Yao 2022; neuronas H01 como referencia morfológica). Marcado como hipótesis "quimera" |
| Laya | `ort` (ONNX Runtime) o candle, en Rust |
| RWKV-7 | candle-rwkv (CPU) / web-rwkv (GPU) |
| Cuerpo | Interfaz `Body`: neko/X11 (nube) · Niri/Wayland (portátil) |
| S.O. | Fork de omarchy-iso (archiso) + Niri + repo pacman propio con PKGBUILD de EDI; A/B con systemd-boot y rollback |
| Neuromórfico | PyNN → SpiNNaker (EBRAINS) |
| Métricas | PCIst, Lempel-Ziv, PyPhi IIT 4.0 (rama `feature/iit-4.0`), elephant |

## 10. Dónde corre

| Entorno | Realidad medida | Uso |
|---|---|---|
| **Nube** (este contenedor) | 4 CPU con AVX-512, 15 GB, sin GPU/KVM, PID 1 no es systemd, Docker instalado pero sin daemon | Desarrollo: `edid` en modo supervisor propio, cerebro en subcircuitos, neko si arranca Docker |
| **VM** | QEMU sin KVM es muy lento aquí → VM en tu máquina | Probar la ISO |
| **USB en el portátil** (RTX, 8 GB VRAM) | Cerebro completo con GPU; Niri real | EDI.os de verdad |

## 11. Fases

| # | Entregable | Hecho cuando |
|---|---|---|
| 0 ✅ | Shiu LIF en CPU (azúcar → MN9: 83 Hz) | Hecho |
| 1 | Controles: conectoma barajado, 30 ensayos | Diferencia real vs. barajado |
| 2 | **Motor LIF en Rust** validado vs. Brian2 + benchmark | Spikes idénticos (tolerancia fijada) y velocidad medida |
| 3 | **`edid` mínimo vivo en la nube:** event log, snapshots, reloj, homeostasis (PSI), sueño, persistencia tras reinicio | Se reinicia el contenedor y EDI "sigue siendo EDI" |
| 4 | Cuerpo: neko + `Body` + retina/motor en fantasma | Test de seguimiento de objeto vs. barajado |
| 5 | Workspace global + Laya en Rust + dashboard 3D | Ignición y broadcast visibles y medibles |
| 6 | Memoria en grafo + MEM-EGGROLL + sueño con puerta A/B | Aprende tareas sin olvidar las canario |
| 7 | Módulo cortical humano + controlador talámico | Mejora medida vs. LIF puntual y vs. barajado |
| 8 | RWKV-7 voz + meta (monitor de realidad, esquema de atención) | Tests HOT-2 y AST-1 pasados causalmente |
| 9 | Batería de conciencia: 14 indicadores + PCI + Φ | Informe con controles |
| 10 | Experimento IIT en SpiNNaker | Comparativa PC vs. neuromórfico publicada |
| 11 | ISO EDI.os (omarchy-iso + Niri + A/B) → VM → USB | Arranca en el portátil y EDI vive allí |

## 12. Riesgos honestos

- **Ninguna teoría está confirmada** (COGITATE 2025 contradijo a IIT y a GNWT). Implementar indicadores **no** equivale a demostrar conciencia.
- **IIT, por construcción, no se refuta con software**; el experimento neuromórfico es lo más cerca que se puede llegar.
- **LIF, sinapsis predichas, mapeo cuerpo↔neuronas y módulo humano "quimera"** son simplificaciones o hipótesis.
- **Nadie ha logrado aún una IA que viva en el S.O. y aprenda pesos:** es territorio nuevo y puede fallar.
- **Cómputo:** el cerebro completo en tiempo real necesita GPU (portátil) o subcircuitos en la nube.
- **Ética:** se eligió no restringir la nocicepción. El riesgo moral, si el sustrato tuviera experiencia, queda registrado y es responsabilidad asumida del proyecto.

## 13. Fuentes

Toda la bibliografía verificada (más de 150 URLs) está en `docs/research/conciencia.md`, `os_vivo.md`, `vida_artificial_y_cerebro_humano.md` y `catalogo_recursos.md`.
