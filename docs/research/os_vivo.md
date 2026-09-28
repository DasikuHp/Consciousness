# EDI.os: cómo hacer un S.O. "vivo" que sea un sistema y no un amalgama

> Investigación de ingeniería · 2026-09-28 · Estado: propuesta, no implementado.
> Método: búsqueda y lectura web. Cada afirmación externa lleva su URL (§9). Lo que es **inferencia o propuesta nuestra** va marcado como tal. Lo que **no pude verificar** va marcado como ⚠️.
> Contexto: `goal.md` (BrainOS/EDI.os), `docs/research/catalogo_recursos.md`.

---

## 0. Veredicto en una página

1. **Nadie ha construido todavía un "S.O. con una IA que vive dentro" que funcione de verdad.** Lo que existe son (a) *kernels de agentes* en espacio de usuario (AIOS, Letta/MemGPT), (b) *capas de agente* sobre un S.O. normal (UFO² en Windows, Windows Agent Workspace, ANOLISA de Alibaba, MCP en SLES 16) y (c) *agentes de uso de ordenador* (Agent S3, UI-TARS-2, modelos frontera en OSWorld). Los que fracasaron (Rabbit R1, Humane, 01 Light) prometieron autonomía que el producto no daba. Ninguno hace aprendizaje continuo en local.
2. **El riesgo de "amalgama" no lo quita tener un proceso o diez.** Lo quita tener **un único dueño del estado, del tiempo, del esquema y del ciclo de vida.** Propuesta: **`edid`**, un solo daemon Rust que es el "núcleo" de EDI.os. Enlaza *dentro de sí* Laya, RWKV-7, memoria, homeostasis, políticas y el registro de eventos, y solo saca de su proceso lo que **debe** estar aislado: el cerebro (Python/Brian2 hoy), el cuerpo (sandbox neko o sesión Niri) y el compositor.
3. **Percepción estructurada sin trucos de píxeles:** sí se puede, pero por partes. AT-SPI (árbol de accesibilidad por D-Bus) + IPC de Niri (ventanas, foco, geometría) + captura (wlr-screencopy) + entrada virtual (wlr-virtual-pointer / zwp-virtual-keyboard). Hay dos problemas reales: bajo Wayland AT-SPI devuelve coordenadas **relativas a la ventana**, y Niri 26.04 **todavía no** trae `ext-image-copy-capture` (hay PRs abiertas). En la nube el cuerpo es neko, que es **X11**, así que ahí se usa AT-SPI + XTest/xdotool. Hace falta un trait `Body` con dos backends.
4. **El cerebro en Rust es realista.** Ya existen motores del modelo LIF de Shiu fuera de Brian2 que igualan a Brian2 spike a spike (MLX/Metal, Rust/Metal, PyTorch CSR). Recomiendo un **motor LIF en Rust (CPU, CSR, event-driven)** como runtime y **Brian2 como verdad de referencia**, que es el mismo patrón que siguió flyBrain. ⚠️ No hay benchmark publicado de un motor Rust **en CPU** para el cerebro completo: hay que medirlo.
5. **Laya y RWKV-7 caben en Rust.** Laya es ModernBERT-large (421M) + cabeza propia, y candle-transformers tiene ModernBERT. RWKV-7 está en candle-transformers, web-rwkv (GPU/wgpu), rwkv.cpp y llama.cpp (CPU).
6. **Lo "vivo" es ingeniería de procesos:** bucle perpetuo con tres ritmos, ciclo vigilia/sueño que consolida fuera de línea, interocepción real (PSI de `/proc/pressure/*`, que funciona en este contenedor), watchdog, snapshots propios y actualizaciones A/B con rollback, tanto del S.O. (systemd-boot boot counting) como de los **pesos**.
7. **Irreal a corto plazo:** Niri dentro del Docker de la nube sin GPU, el ISO arrancable probado en QEMU sin KVM a velocidad útil, y el aprendizaje continuo de Laya 421M en CPU. **Realista ya:** el MVP "vivo" en la nube (§8.5): `edid` + cerebro (subcircuito) + neko + memoria + sueño + homeostasis + dashboard.

---

## 1. Estado del arte: S.O. "nativos de IA" y agentes de S.O.

### 1.1 Tabla comparativa

| Proyecto | Qué es realmente | Mecanismos clave | Qué funcionó | Qué falló / límites |
|---|---|---|---|---|
| **AIOS** (Rutgers, agiresearch) | "Kernel" de agentes **en espacio de usuario** (Python), no un S.O. | Scheduler (FIFO, Round-Robin) de *syscalls de LLM*; context manager con snapshot/restore del contexto; memory manager; storage manager; tool manager; access manager. SDK aparte (Cerebrum). Modos local/remoto; VM Controller + MCP Server para uso de ordenador; terminal con sistema de archivos semántico | Hasta 2,1× más rápido sirviendo agentes de varios frameworks; paper en COLM 2025 | Última versión etiquetada v0.2.2 (12-mar-2025). Es un *multiplexor de LLM*, no gestiona el hardware. Los "modos virtual/personal kernel" siguen "en desarrollo" |
| **MemGPT → Letta** (Berkeley, 2023→2024) | Memoria virtual para LLM: contexto = RAM, archivo = disco | El agente pagina entre *core memory* (en contexto) y *archival/recall* (fuera). **Sleep-time compute** (abr-2025): agentes que reorganizan la memoria en tiempo ocioso, de forma asíncrona | La analogía con el S.O. funciona para la memoria. El sueño asíncrono mejora la latencia y la calidad de la memoria | Solo contexto: **no** hay aprendizaje de pesos |
| **LLM OS de Karpathy** (10-nov-2023) | Metáfora: LLM = CPU, contexto = RAM, embeddings = sistema de archivos, navegador/terminal = periféricos | Marco conceptual | Ha ordenado cómo piensa todo el sector (uso de ordenador = periféricos) | No es software |
| **OS-Copilot / FRIDAY** (2024) | Agente generalista que se auto-mejora acumulando herramientas | Aprende "skills" (código) de tareas previas | +35 % sobre métodos previos en GAIA; se auto-mejora en Excel/PowerPoint con poca supervisión | Solo biblioteca de skills (código); no aprende pesos |
| **UFO² / UFO³** (Microsoft) | "Desktop AgentOS" para Windows | HostAgent + AppAgents; percepción **híbrida UIA (accesibilidad) + visión**; acciones híbridas GUI/API (Win32, COM); escritorio *Picture-in-Picture* para no molestar al usuario; Galaxy orquesta varios dispositivos con DAG | Usar la API del S.O. (UIA/COM) antes que los píxeles | Windows solamente |
| **Windows 11 agentic** (2025-26) | Agent Workspace: sesión paralela con **cuenta propia por agente**, MCP nativo, Copilot Actions | Identidad separada, espacio aislado, registros de auditoría | El modelo de seguridad (identidad por agente, aislamiento, auditoría) | Microsoft avisa de **XPIA** (inyección de instrucciones a través de UI o documentos que puede llevar a instalar malware). Viene desactivado por defecto; la visión completa no llega a disponibilidad general hasta 2027 |
| **ANOLISA** (Alibaba Cloud Linux 4 Agentic Edition, v0.5 29-may-2026) | Distro Linux "agent-first" | `cosh` (Copilot Shell): shell compatible con bash + lenguaje natural; "OS Skills" (manuales del S.O. legibles por agentes); AgentSecCore; AgentSight (observabilidad); snapshots de workspace y rollback | Es lo más parecido a una distro con agente integrado. Código abierto en github.com/alibaba/anolisa | Orientado a servidores y agentes LLM en la nube; sin escritorio ni aprendizaje |
| **SLES 16 / Fedora** | MCP host/servidor en el S.O. (SLES 16, preview); linux-mcp-server (Fedora) | Exponen el S.O. como herramientas MCP | Integración estándar | Hay críticas públicas ("mala idea gestionar Fedora con IA", OSnews) |
| **Agent S3** (Simular) | Framework abierto de uso de ordenador | **bBoN (Behavior Best-of-N)**: varias ejecuciones, cada una resumida como narrativa de comportamiento, y un juez elige la mejor | 72,6 % en OSWorld, por encima del nivel humano (~72 %) | Cuesta N ejecuciones |
| **UI-TARS-2** (ByteDance) | Modelo nativo de GUI entrenado con RL multi-turno | Volante de datos, entorno híbrido GUI + terminal + FS, sandbox unificado | 47,5 OSWorld; 50,6 WindowsAgentArena; 73,3 AndroidWorld | Hoy superado por modelos generalistas |
| **Rabbit R1 / Humane Pin / 01 Light** | Hardware con "Large Action Model" | — | — | La demo prometió lo que el producto no hacía; batería y temperatura; "¿qué hace esto mejor que un móvil?". Open Interpreter canceló el 01 Light y lo convirtió en una app |

### 1.2 Estado de OSWorld (sept. 2026)

- **OSWorld-Verified** (versión reparada de jul-2025, 369 tareas, evaluación por ejecución). Primeros puestos a 27-sep-2026 según BenchLM: Qwen3.8 Max 86,1 %, Claude Fable 5 85 %, Claude Mythos 5 85 %, Qwen3.8-27B 84,3 %, Claude Opus 4.8 83,4 %… El mejor de pesos abiertos es **Holo3-35B-A3B, con 82,6 %**. La mayoría de filas son **auto-reportadas** por cada proveedor.
- Existe **OSWorld 2.0** con su propio leaderboard (Steel.dev).
- **Dato clave para EDI.os:** en el paper original de OSWorld, **el árbol de accesibilidad solo rinde más que la captura sola, y la combinación a11y + captura es lo mejor** (a11y 18–21 %, captura 5–15 %, a11y+captura 18–26 %, set-of-marks 13–19 %, con los modelos de entonces).

### 1.3 Lecciones para EDI.os

| Lección | Evidencia | Cómo se aplica |
|---|---|---|
| La estructura rinde más que los píxeles; lo híbrido rinde más que ambos | OSWorld (a11y+captura), UFO² (UIA+visión) | Percepción = árbol a11y + ventanas + píxeles |
| Aislar al agente en su propio espacio e identidad | Windows Agent Workspace, UFO² PiP | El cuerpo de EDI es un escritorio **separado** (neko / sesión Niri propia) con un usuario Unix propio |
| La inyección de instrucciones desde la UI es el riesgo nº 1 | Aviso XPIA de Microsoft | Laya valida; hay escalera de autonomía, capacidades y registro de todo |
| Consolidar la memoria en tiempo ocioso | Letta sleep-time, sueño de Tadros et al. | Ciclo vigilia/sueño (§4) |
| No prometer autonomía antes de medirla | Rabbit/Humane | Escalera fantasma → validado → autónomo (ya está en `goal.md`) |
| Nadie tiene aprendizaje continuo de pesos en un S.O. | Ninguno de la tabla lo hace | Es terreno nuevo; tratarlo como experimento con controles |

---

## 2. Percepción y acción a nivel de S.O. (no a nivel de píxel)

### 2.1 Canales disponibles

| Canal | Qué da | Estructurado | En neko (X11, nube) | En Niri (Wayland, VM/USB) | Crate Rust |
|---|---|---|---|---|---|
| **AT-SPI2** (D-Bus `org.a11y.atspi`) | Árbol de objetos: rol, nombre, estado, acciones, texto, eventos | ✅ | ✅ (apps GTK/Qt/Chromium/Firefox con a11y activada) | ✅ pero con **coordenadas relativas a la ventana** (ver 2.2) | `atspi` (Odilia, puro Rust sobre zbus) |
| **Niri IPC** (`niri msg --json`, socket) | Ventanas (id, pid, app_id, título, workspace, foco, geometría), acciones (`focus-window --id`) y stream de eventos | ✅ | — | ✅ | `niri-ipc` (crate del proyecto) ⚠️ no verificado aquí |
| **wlr-screencopy v3** | Captura de salida | píxeles | — | ✅ (desde niri 0.1.8) | `wayland-client` + `wayland-protocols-wlr` |
| **ext-image-copy-capture-v1** (estándar nuevo, por salida y por ventana) | Captura por *toplevel* | píxeles | — | ❌ en niri 26.04 (0 símbolos, issue cua #3735, 11-sep-2026); **PRs abiertas**: #4554 (salidas/cursores) y #4599 (toplevel) | idem |
| **ext-foreign-toplevel-list** | Lista de ventanas estándar | ✅ | — | ✅ (implementación propia de niri, no la de Smithay) | idem |
| **zwlr-virtual-pointer / zwp-virtual-keyboard** | Ratón y teclado virtuales | acción | — | ✅ (virtual-pointer desde 25.02) | idem |
| **libei / EIS** | Entrada emulada distinguible por el compositor (control de acceso fino) | acción | — | ⚠️ no encontré soporte de EIS en niri | bindings C |
| **XTest / xdotool** | Ratón y teclado en X11 | acción | ✅ (neko es X11) | — | `x11rb` (XTest) |
| **Captura X11** (XShm/XGetImage) | Píxeles | píxeles | ✅ | — | `x11rb` |
| **KeyboardMonitor D-Bus** de niri | Para lectores de pantalla (Orca) | — | — | ✅ desde 25.08 | zbus |

**Resultados reales con niri** (issue trycua/cua #3735, niri 0.26.04): captura global ✅ y enumeración de ventanas ✅. Fallan la captura por ventana (imagen 0×0), el teclado y el ratón dirigidos a una ventana concreta (Electron rechazó la inyección en segundo plano) y la activación/foco "atestiguada". El reporter concluye que **faltan mecanismos de atestiguar el objetivo, no protocolos**, y que la IPC de niri ya da lo necesario (`focus-window --id`) **si el driver la usa**.
→ **Para EDI.os (inferencia):** la secuencia correcta en Niri es *IPC → enfocar por id → verificar foco por IPC → inyectar con virtual-pointer/keyboard en coordenadas de salida = geometría de la ventana (IPC) + offset del objeto (AT-SPI)*. Así se resuelve también el problema de coordenadas de 2.2.

### 2.2 Problemas conocidos (no ocultar)

- **Coordenadas AT-SPI bajo Wayland:** `GetExtents(SCREEN)` devuelve coordenadas **relativas a la ventana**, porque en Wayland el cliente no conoce su posición. La nueva arquitectura de accesibilidad de GNOME asume coordenadas relativas a la superficie. Solución: sumar la geometría de la ventana que da la IPC del compositor (issue kwin-mcp #51 con el mismo diagnóstico).
- **Escalado fraccional:** los marcos de coordenadas de la captura y de la entrada no coinciden (cua #3061). Recomendación: escala 1.0 en la salida del cuerpo.
- **La accesibilidad de niri depende de EGL con aceleración hardware** (documentación de niri). Y **Alacritty (terminal por defecto de Omarchy) no es accesible**: la documentación de niri recomienda GNOME Console/Terminal. En EDI.os, el terminal del cuerpo debe exponer AT-SPI.
- **neko** es un navegador/escritorio virtual en Docker transmitido por WebRTC, con imágenes XFCE/KDE. La documentación **no menciona Wayland**. Se conecta a un servidor X, así que en la nube el cuerpo es X11.

### 2.3 Diseño recomendado: un único trait `Body`

```rust
// propuesta (inferencia nuestra)
trait Body {
    async fn snapshot(&self) -> Percept;            // árbol a11y + ventanas + frame (opcional)
    async fn act(&self, a: Action, cap: &Capability) -> ActOutcome; // verificada
    fn events(&self) -> impl Stream<Item = UiEvent>; // foco, ventana nueva, cambio de texto
}
struct X11NekoBody  { atspi, x11rb_xtest, x11_capture }      // nube
struct NiriBody     { atspi, niri_ipc, screencopy, vptr, vkbd } // VM / USB
```

- **`Percept`** es un tipo único, versionado, con tres niveles: `a11y: Tree` (roles, nombres, acciones), `windows: Vec<Win>` y `frame: Option<Frame>` (reducida a rejilla de omatidios para el cerebro, §3.4).
- **Cada modelo consume lo suyo:** el **cerebro** consume píxeles (la mosca ve luz, tiene sentido biológico), **Laya** consume el árbol (texto con opciones, que es su formato natural: `choice/score/booleano`) y **RWKV** consume el resumen.
- **La acción siempre se verifica:** después de actuar se vuelve a hacer `snapshot` y se comprueba el efecto (el foco cambió, el texto apareció). La verificación va al registro de eventos.

---

## 3. Un sistema coherente y no un amalgama

### 3.1 Qué hace que algo sea un amalgama (diagnóstico)

| Síntoma de amalgama | Antídoto |
|---|---|
| Cada servicio tiene su propio estado, config y logs | **Un único registro de eventos** + un único árbol de config tipado |
| JSON ad hoc sobre HTTP entre piezas | **Un IDL único** (esquema versionado) para todos los mensajes |
| Cada pieza tiene su reloj | **Un único reloj lógico** (tick) que dueña `edid` |
| Arrancar/parar a mano en orden | **Un supervisor** con grafo de dependencias, salud y reinicio |
| Permisos implícitos (todo puede todo) | **Capacidades explícitas** emitidas por `edid` |
| No se puede reproducir un día | **Event sourcing + snapshots + semillas** = replay determinista |
| Cada modelo decide solo | **Un único ciclo de decisión** (percibir → proponer → validar → actuar → aprender) |

**Principio (inferencia nuestra):** *el número de procesos es una decisión de aislamiento, no de arquitectura*. Todo lo que es de confianza y de Rust va **en proceso** dentro de `edid`, como módulos (crates del workspace). Fuera de proceso va solo lo que es (a) no-Rust, (b) no confiable o (c) otro dominio de fallo: el cerebro Python, el sandbox y el compositor.

### 3.2 Opciones de IPC (y cuál usar para qué)

| Tecnología | Tipo | Pros | Contras | Uso en EDI.os |
|---|---|---|---|---|
| Canales tokio en proceso | memoria compartida del proceso | Coste ~0, tipos de Rust | Solo dentro de `edid` | **Bus interno** entre módulos de `edid` |
| **iceoryx2** | Memoria compartida zero-copy (pub/sub, eventos, req/resp, *blackboard*) | Latencia constante con cualquier tamaño de carga; núcleo en Rust; **bindings Python oficiales desde v0.7** (`pip install iceoryx2`, vía PyO3) | Hay que diseñar tipos `#[repr(C)]` / POD | **Plano de datos cerebro ⇄ edid** (vectores de spikes y tasas, frames) |
| **zbus** (D-Bus) | RPC y señales | Puro Rust, async, integra con el escritorio (AT-SPI también es D-Bus, niri expone D-Bus para a11y) | No sirve para mucho volumen | **Plano de control y API pública** `org.edios.Edi1` (CLI, dashboard, systemd, panel de Niri) |
| **Cap'n Proto RPC** (capnp-rpc) | RPC con *capacidades de objeto* | Las referencias son capacidades: se pueden delegar y revocar | Implementación "nivel 1"; otra toolchain | Opcional: si se quiere seguridad por capacidades entre procesos. Alternativa: capacidades como tokens firmados dentro de D-Bus |
| Unix socket + bincode/postcard | stream | Simple | Hay que inventar el protocolo | No (así nace un amalgama) |

**Recomendación:** dos planos. **Control** por D-Bus/zbus, con una interfaz pública, introspectable y con permisos. **Datos** por iceoryx2, sin copia, para el cerebro y los frames. Un **único crate `edi-schema`** define los tipos de ambos (y genera los stubs Python del cerebro). Esa es la columna vertebral que evita la amalgama.

### 3.3 El cerebro: embebido, proceso aparte o port a Rust

| Opción | Cómo | Pros | Contras | Veredicto |
|---|---|---|---|---|
| **A. PyO3 embebido** en `edid` | `pyo3` con `auto-initialize`; Python libre de GIL (3.14t) soportado desde PyO3 0.23 | Un solo proceso | Un fallo de Brian2 (codegen C++, OOM) **tira el núcleo**; el GIL o el adjuntar hilos complica el tiempo real; hay que enlazar libpython | ❌ para el cerebro (sí para scripts de análisis puntuales) |
| **B. Proceso aparte + iceoryx2** | `edi-brain` (Python/Brian2) publica y suscribe por iceoryx2 y `edid` lo supervisa | Aislamiento de fallos; Brian2 intacto como verdad | Otro proceso (pero con un único esquema) | ✅ **Fase actual** |
| **C. Motor LIF nativo en Rust** (`edi-lif`) en proceso | CSR ordenado por origen, actualización exacta en forma cerrada de `v` y `g`, umbral estricto, dt = 0,1 ms, acumulación en enteros (determinista) | Sin Python en caliente, snapshots triviales (arrays), SIMD (AVX-512 disponible en este contenedor), multihilo, determinista | Hay que **validar** contra Brian2 | ✅ **Objetivo**: runtime en Rust, Brian2 como referencia |

**Precedentes de la opción C (verificados):**
- **mlx-lif-engine / drosophila-brain-mlx:** 127.400 neuronas y 14,7 M aristas (FlyWire v630). Da **0,29 s por segundo biológico en M4 Pro** frente a 62,6 s de la referencia Brian2 en la misma máquina (en otra medición, 2,07 s). Mismo orden de operaciones que Brian2 (actualización exacta, umbral estricto `v > −45 mV`, dt 0,1 ms), CSR por origen y conteos int32 con el factor 0,275 mV aplicado después, para que el resultado no dependa del orden de los hilos. **Validación:** un subgrafo de 800 neuronas coincide *spike a spike* con el `SpikeMonitor` de Brian2; en el experimento publicado las diferencias son de |z| ≤ 1,9 en 30 ensayos. **Lección que ellos mismos anotan:** su motor pierde un 13,9 % con carga en la CPU porque reconstruye el grafo MLX en el host, mientras que **el competidor en Rust pierde solo un 1,1 %**.
- **flyBrain (mehrantsi):** runtime de producción en **Rust + Metal** (solo Apple, sin backend de CPU), ~0,38 s por segundo biológico en M3 Max. Python queda como carril independiente de verificación (Brian2/NumPy/MLX) y como compilador offline Parquet → CSR (`row_ptr`, `destinations`, `signed_counts` en .npy con hashes revalidados). Tiene puente sensorial a 500 Hz hacia MuJoCo. MIT.
- **fly-brain (eonsystems):** Brian2 (CPU), Brian2CUDA, PyTorch con **CSR disperso**, NEST GPU y GeNN. Brian2 CPU se toma como verdad de referencia.
- **Crates Rust de SNN genéricos:** `oxicuda-snn`, `neuromod`, `neuralos-snn` (no_std, punto fijo, AVX2), `SpikingNN_Rust`, `oldies-brian`. Sirven de referencia; para el conectoma completo conviene un motor propio y mínimo como los anteriores.

⚠️ **Falta un número:** no encontré ningún benchmark publicado de un motor LIF en **CPU** en Rust para el cerebro completo. Nuestra medida actual con Brian2 CPU es ~8 s por segundo biológico (`goal.md`). Estimación (inferencia): con CSR event-driven y actividad dispersa podría bajar un orden de magnitud en 4 núcleos, pero **hay que medirlo** antes de prometer tiempo real.

`goal.md` dice que el cerebro "no se reescribe en Rust". Propuesta compatible: **el modelo sigue siendo el de Shiu** (Brian2 = especificación ejecutable). Rust es solo un *runtime* verificado contra él, y el conectoma y los parámetros son los mismos (hash).

### 3.4 Laya y RWKV-7 en Rust

| Modelo | Hecho verificado | Ruta Rust | Riesgo |
|---|---|---|---|
| **Laya** (convaiinnovations) | Backbone **ModernBERT-large**, 421M (la variante multilingüe usa mmBERT-base, 322M) + cabeza propia (2 capas transformer + scorer de marcadores de opción). Tipos `choice`/`score`/booleano con probabilidad calibrada (ECE 0,213 → 0,081 tras ajustar la temperatura). ~33–40 ms en GPU T4. Apache-2.0. Extra `laya[onnx]` | (1) **ONNX + `ort` 2.0** (rc.12, "listo para producción, API no estable"). (2) **candle-transformers** trae `modernbert.rs`, pero la cabeza hay que portarla a mano | ⚠️ Latencia en CPU sin medir: 421M en fp32 son ~1,7 GB de pesos y probablemente cientos de ms por pregunta en 4 núcleos (inferencia). Conviene cuantizar a int8 y agrupar preguntas en lotes |
| **RWKV-7 0.1B** | candle-transformers incluye RWKV v7 "Goose" (x070). **web-rwkv** (WebGPU/wgpu, v4–v7, int8/nf4, *hooks* para LoRA dinámico). **rwkv.cpp** (CPU, v4–v7) y **llama.cpp** (kernel `GGML_OP_RWKV_WKV7` en CPU/CUDA/Vulkan/Metal) | Nube sin GPU: **candle en CPU**. Portátil: **web-rwkv** (Vulkan). Para EGGROLL hace falta inyectar la perturbación `ABᵀ` en el forward: en candle es código propio y en web-rwkv se hace con *hooks* | Bajo. 0,1B es pequeño |

### 3.5 Seguridad por capacidades y sandboxing

| Capa | Herramienta | Qué aísla | Nota |
|---|---|---|---|
| Unidades del S.O. | directivas de sandbox de systemd (`ProtectSystem`, `PrivateNetwork`, `SystemCallFilter`, `DynamicUser`…) | cada proceso de EDI | Solo en VM/USB (en la nube no hay systemd, §3.7) |
| Proceso cerebro | **Landlock** (crate `landlock`, sin privilegios; ABI v6 con Linux ≥ 6.12) + seccomp | FS y red del cerebro: solo lee el conectoma y solo habla por iceoryx2 | El cerebro no necesita red |
| Herramientas que lanza EDI | **bubblewrap** (namespaces sin privilegios + seccomp; es el runtime de Flatpak) | comandos | No está instalado en este contenedor |
| Cuerpo | neko en Docker / sesión Niri con **usuario Unix propio** para el agente | escritorio | Mismo patrón que Windows Agent Workspace |
| Acciones | **Capacidades** emitidas por `edid` (`Cap{scope: app|window, verbs, ttl, autonomy_level}`) | qué puede hacer cada acción | La escalera de autonomía se implementa **aquí**. Sin capacidad solo hay dibujo fantasma |

### 3.6 Estado: event sourcing, snapshots y replay

- **Registro de eventos append-only** (propuesta): `Percept`, `Proposal`, `LayaVerdict`, `Action`, `Outcome`, `Reward`, `Interocept`, `SleepPhase`, `ParamUpdate`… cada uno con `tick`, `seed` y `hash`. Almacén: **redb** (KV embebido, puro Rust, ACID, crash-safe, B+tree copy-on-write, formato de archivo estable). Los blobs grandes (frames) van a archivos segmentados con hash.
- **Snapshots a nivel de aplicación, no de proceso:** estado LIF (`v`, `g`, contadores refractarios, cola de retardos), estado RNN de RWKV, adaptadores low-rank, grafo de memoria y RNG. Son arrays y se serializan en ms.
- **CRIU no se recomienda como mecanismo principal:** necesita privilegios, las librerías deben ser **exactamente las mismas versiones** al restaurar, siempre vuelca el árbol de procesos completo, las conexiones TCP necesitan opciones especiales y "no puede guardar todo el estado". Sirve como herramienta de depuración, no como persistencia.
- **Replay determinista:** snapshot N + eventos desde N + semillas ⇒ la misma trayectoria. Encaja con el principio 5 de `goal.md`. Solo es posible si **todo** el no-determinismo (entrada del cuerpo, reloj, RNG) entra por el registro.

### 3.7 Hecho importante sobre la nube (medido en este contenedor)

| Comprobación | Resultado |
|---|---|
| PID 1 | `process_api` (no systemd) |
| Kernel | 6.18.44 (microVM "fc") |
| cgroups | **v1** (sin `cpu.pressure` por cgroup) |
| PSI global | ✅ `/proc/pressure/{cpu,memory,io}` |
| `/dev/kvm` | ❌ |
| Docker | cliente 29.3.1 instalado; **daemon no arrancado** (sin `/var/run/docker.sock`) |
| qemu, bwrap | no instalados |
| CPU | 4 vCPU con AVX2 y **AVX-512F** |

→ **`edid` debe poder ser supervisor por sí mismo** ("modo contenedor": arranca y vigila a sus hijos, lee PSI global) **y** delegar en systemd en el S.O. real ("modo sistema": units, watchdog, cgroups v2 por unidad). Es el mismo binario con dos backends de supervisión, detrás de un trait `Supervisor`.

---

## 4. Ingeniería del proceso "vivo"

### 4.1 Bucle perpetuo con tres ritmos (propuesta)

| Ritmo | Periodo | Quién | Qué hace |
|---|---|---|---|
| **Reflejo** | 10–100 ms (tick) | cerebro LIF + `Body` | percibir → spikes → intención motora (fantasma o real según la capacidad) |
| **Deliberación** | 0,5–5 s | Laya (+ RWKV) | valida o elige entre opciones del árbol a11y; RWKV escribe una nota |
| **Consolidación** | horas (sueño) | memoria + EGGROLL | repetición, ES sobre adaptadores, olvido, compactación del registro |

El reloj lo lleva `edid`. Si el cerebro va más lento que el tiempo real, el sistema **no finge**: registra `bio_time/wall_time` y el reflejo se degrada (subcircuito o menos neuronas), lo que es una decisión explícita y registrada.

### 4.2 Vigilia y sueño

- **Base empírica:** una fase de sueño con plasticidad hebbiana local y entrada ruidosa **recupera tareas antiguas olvidadas** en redes artificiales (Tadros, Krishnan, Ramyaa, Bazhenov, *Nat. Commun.* 2022). Letta llama a lo mismo, aplicado a la memoria de agentes, *sleep-time compute*.
- **Planificador circadiano (propuesta):** el sueño se dispara por **presión homeostática** (eventos sin consolidar, fatiga = PSI alto sostenido) **y** por ventana horaria (cuando el usuario no usa la máquina). Si el usuario usa el portátil, EDI no entrena. Fases: **N1** compactar el registro → **N2** repetición priorizada (IER/SER) → **N3** EGGROLL sobre adaptadores con *fitness* = rendimiento nuevo + penalización por olvido (conjunto *canario*) → **REM** recombinación/contrafactuales (hipótesis de `goal.md` §8) → **despertar** con puerta de promoción (4.4).

### 4.3 Interocepción y homeostasis

| Señal | Fuente | Disponible en nube | Uso |
|---|---|---|---|
| Presión de CPU/memoria/IO | PSI `/proc/pressure/*` (some/full, avg10/60/300); por cgroup en v2 (`cpu.pressure`, `memory.pressure`) | ✅ global | "Fatiga": bajar el ritmo o subcircuito; disparar sueño |
| Umbrales con `poll()` | Triggers PSI (sin privilegios, ventana múltiplo de 2 s) | ✅ probablemente | Eventos de interocepción en lugar de sondeo |
| Memoria | cgroup `memory.current/max` (v2) | parcial (v1) | Presupuesto de RAM (15 GB) |
| Temperatura | `/sys/class/thermal`, NVML en el portátil | ❌ nube | Freno térmico (el portátil "no debe sufrir") |
| OOM | **systemd-oomd** (PSI + cgroups v2, `ManagedOOMMemoryPressure=`) | ❌ nube | En el S.O. real: matar primero el entrenamiento y nunca `edid` |

Propuesta (**hipótesis**, se marca así): mapear la interocepción a **neuromoduladores** del modelo (p. ej. ganancia global u octopamina-like) como entrada *explícita* y registrada, nunca tocando el conectoma.

### 4.4 Persistencia, watchdog y auto-actualización segura

- **Watchdog:** `WatchdogSec=` + `Restart=on-watchdog`. `edid` envía `WATCHDOG=1` cada ½ periodo (crate `sdwd` o `sd-notify`). En modo contenedor, `edid` hace de watchdog de sus hijos, y un script externo mínimo lo reinicia a él.
- **S.O. A/B:** con **systemd-boot boot counting**, las entradas nuevas llevan `+tries` y `systemd-bless-boot` las marca "good" al completar el arranque. Si se agotan los intentos, se vuelve a la anterior. Con **mkosi + systemd-sysupdate** hay imágenes A/B con verity y UKI (ejemplo: `arch-image-based`, "un arranque fallido hace rollback solo").
- **Pesos A/B (propuesta, la misma idea aplicada al aprendizaje):** cada generación de adaptadores es una "entrada de arranque" con `tries`. Solo se **promueve** si pasa una puerta: canarios sin olvido, tests del sandbox ≥ los de la generación anterior y conectoma barajado como control. Si se degrada en vigilia, **rollback automático**. Se guardan N generaciones.
- **Arranque:** `edid` restaura el último snapshot "bueno" y aplica el registro posterior.

---

## 5. Construir una imagen de S.O. real y distribuible

### 5.1 Omarchy y Okimarchy: cómo se instalan

| Hecho | Fuente |
|---|---|
| Omarchy 3: **"el ISO de Omarchy es la única forma soportada de instalar"**. El ISO instala Arch, instala los paquetes de Omarchy **desde un mirror incluido en el ISO**, configura en chroot, crea el usuario y ejecuta `omarchy-setup-user` | manual de Omarchy / omarchy-iso |
| `omarchy-iso` se construye con **archiso** (submódulo), con `./bin/omarchy-iso-make`; canales `--edge/--dev/--rc`, `--local-source` | github.com/omacom/omarchy-iso |
| **Instalación desatendida:** un disco `cidata` (cloud-init NoCloud) con `user_configuration.json` y `user_credentials.json` salta el configurador. Pensado para Proxmox/libvirt | idem |
| Instalación manual sobre Arch: todavía existe, pero "no es para la mayoría" | manual |
| **Okimarchy** = fork de Omarchy con elección de **Hyprland o Niri** (o ambos, con `okimarchy-wm-switch`) y config de Niri modular (`bindings.kdl`, `layout.kdl`…). Se instala **por script** sobre Arch: `curl …/boot.sh \| OMARCHY_REPO="cristian-fleischer/okimarchy" bash`. "Compatible con Omarchy 3" | github.com/cristian-fleischer/okimarchy |

⚠️ **Riesgo:** Okimarchy sigue el camino del *script*, mientras que Omarchy 3 ya empuja el *ISO + paquetes*. Hornear Okimarchy en un ISO exige: (a) **empaquetar** (PKGBUILD) lo que el script hace, o (b) ejecutar el script en chroot durante la construcción. (a) es lo "no amalgama". (b) es frágil (el script pide datos interactivos y descarga de la red).

### 5.2 Dos caminos de imagen

| | **archiso (estilo omarchy-iso)** | **mkosi (imagen inmutable)** |
|---|---|---|
| Resultado | ISO live + instalador | Imagen de disco/UKI firmable, A/B con sysupdate |
| Actualizaciones | pacman (mutable) | A/B atómicas + rollback de arranque |
| Encaje con Omarchy | **Alto** (es lo que hace Omarchy) | Bajo (Omarchy asume un sistema mutable) |
| Encaje con "vivo + auto-update seguro" | Medio (usar snapshots btrfs) | **Alto** |
| En Docker | `--privileged` (loop devices, mount); pacstrap necesita root | Igual o más privilegios |

**Recomendación (inferencia):** empezar con **archiso**, forkeando `omarchy-iso`, con un **repo pacman propio** `edios` (paquetes `edid`, `edi-brain`, `edi-models`, `okimarchy-niri`) y autoinstalación `cidata` para VM. Pasar a mkosi/A-B solo si la auto-actualización del S.O. se vuelve crítica. Los **pesos** se actualizan A/B por su cuenta (4.4) con cualquiera de los dos.

### 5.3 Dónde se prueba

| Entorno | Realidad |
|---|---|
| **Docker en la nube** | El daemon no está arrancado aquí. Si se levanta, mkarchiso necesita `--privileged` y nodos loop. ⚠️ No verificado que esta microVM lo permita. neko sí es el caso de uso típico de Docker |
| **Niri en la nube** | Niri tiene backend **headless** (pensado para tests). Las salidas virtuales para sesiones headless/VNC están en la **PR #3800, abierta** (sept-2026) y en forks. Niri usa Smithay con renderer pixman disponible, pero su doc de a11y exige EGL con hardware. ⚠️ Niri real en este contenedor sin GPU = **experimental**. Alternativa probada para Wayland headless: sway con `WLR_BACKENDS=headless WLR_RENDERER=pixman` + wayvnc |
| **QEMU sin KVM** | TCG (emulación por traducción binaria). MTTCG da un hilo por vCPU, pero es mucho más lento que KVM. Útil para "¿arranca el ISO?" y no para usar EDI dentro. qemu no está instalado aquí |
| **VM con KVM** (otra máquina) | Prueba real del ISO + autoinstalación cidata |
| **USB en el portátil** | Final. NVIDIA 8 GB: web-rwkv (Vulkan), Laya en GPU y cerebro en GPU opcional |

---

## 6. Aprendizaje continuo en el dispositivo sin olvido catastrófico

### 6.1 Estado 2025–2026

| Método | Qué es | Olvido | ¿CPU viable para EDI? |
|---|---|---|---|
| **EGGROLL** (nov-2025) | ES con perturbaciones low-rank `ABᵀ`, coste O(r(m+n)); funciona con r = 1; ~91 % del throughput de inferencia por lotes; probado en fine-tuning de LM y en RNN de enteros | Depende del *fitness* (se puede penalizar el olvido dentro de él) | ✅ en RWKV 0,1B y en ganancias del cerebro: solo pasadas hacia delante |
| **"EGGROLL, Unrolled" / LOO-ROLL** (sept-2026) | Prueba que EGGROLL recupera el gradiente exacto en cuadráticas a cualquier rango. LOO-ROLL (leave-one-out) usa 1 evaluación en vez de 2 antitéticas por dirección y reduce a la mitad el error del estimador | — | ✅ **mejora directa y barata**: adoptarla |
| **SEAL** (MIT, 2025) | El modelo genera "self-edits" y aprende a generarlos con RL; actualiza con LoRA | **Sí olvida** con ediciones repetidas (reconocido por los autores) | Parcial: es la idea de `goal.md` §8.5 (self-edits + EGGROLL) |
| **LoRA** | Adaptadores low-rank | 2025–26: **LoRA sola no evita el olvido** en continual learning | Adaptadores por skill + enrutado + repetición, sí |
| **EWC** (Kirkpatrick, PNAS 2017) | Penalización cuadrática ponderada por la Fisher diagonal | Reduce | En ES: añadir `−λ Σ F_i (θ_i−θ*_i)²` al fitness (inferencia) |
| **Repetición** (replay) | Mezclar episodios antiguos | Reduce (la más robusta en la práctica) | ✅ (grafo de memoria + canarios) |
| **Sueño** (Tadros 2022) | Fase offline con plasticidad hebbiana + ruido | Recupera tareas olvidadas | ✅ encaja con el SNN |
| **Nested Learning / Hope** (Google, NeurIPS 2025) | Módulos con distintas velocidades de actualización (multiescala) | Menos olvido | Idea de diseño: nuestras tres escalas de tiempo |
| **TTT-E2E** (dic-2025) | Contexto largo como aprendizaje continuo en inferencia | — | Idea afín a RWKV (estado = pesos rápidos) |
| **Continual merging** (ICLR 2026, "Merge before Forget") | Un solo LoRA que se fusiona de forma continua | Reduce | Candidato para consolidar adaptadores en el sueño |

### 6.2 Qué aprende qué en EDI.os (propuesta coherente con `goal.md`)

| Escala | Qué cambia | Dónde | Frecuencia | Reversible |
|---|---|---|---|---|
| **Rápida** (s) | Estado LIF, estado RNN de RWKV, activación del grafo | RAM | cada tick | se resetea |
| **Media** (min–días) | Grafo de memoria (episódico → semántico → procedural) | redb | continua | sí (registro) |
| **Lenta** (días) | Adaptadores low-rank de RWKV, ganancias por tipo celular, temperatura de Laya | archivos versionados A/B | solo en el sueño | sí (A/B) |
| **Nunca** | Conectoma, pesos base de Laya y RWKV | read-only | — | — |

⚠️ **Realismo:** entrenar Laya 421M con ES en 4 CPU es **inviable** a corto plazo (inferencia). Solo se calibra temperatura/umbral (ya está previsto). RWKV 0,1B con EGGROLL es viable a pequeña escala. El cerebro solo evoluciona un genoma pequeño (ganancias), con CMA-ES/EGGROLL.

---

## 7. Arquitectura recomendada para EDI.os

### 7.1 Diagrama

```
                         ┌──────────────── edid (un binario Rust, tokio) ─────────────────┐
  usuario ── D-Bus ─────►│ api   : org.edios.Edi1 (zbus) — autonomía, kill, fantasma, stats  │
  (CLI, panel Niri,      │ clock : tick lógico único + planificador circadiano             │
   dashboard web)        │ log   : registro de eventos (redb) + snapshots + replay         │
                         │ caps  : emisor/verificador de capacidades (escalera autonomía)  │
                         │ homeo : PSI / thermal / RAM → interocepción + presupuestos      │
                         │ super : Supervisor{Container|Systemd} — hijos, salud, reinicio  │
                         │ cortex: Laya (ort ONNX | candle ModernBERT)   ─┐ en proceso      │
                         │ voice : RWKV-7 (candle CPU | web-rwkv GPU)     │ bus interno     │
                         │ memory: grafo ep→sem→proc + recuperación       │ (canales tokio) │
                         │ learn : EGGROLL/LOO-ROLL + replay + A/B pesos ─┘ (solo en sueño) │
                         │ body  : trait Body {X11Neko | Niri} + retina/motor              │
                         └──────┬───────────────────────────┬──────────────────┬───────────┘
                     iceoryx2   │ (spikes, tasas, frames)   │ AT-SPI (zbus)     │ X11/XTest | niri IPC
                                ▼                           ▼  + captura        ▼  + virtual ptr/kbd
                   ┌───────────────────────┐     ┌──────────────────────────────────────────┐
                   │ edi-brain (proceso)   │     │ CUERPO: neko (Docker, X11)  [nube]       │
                   │ fase A: Brian2/Python │     │         sesión Niri usuario `edi` [VM/USB]│
                   │ fase B: edi-lif Rust  │     │ usuario Unix propio, sin acceso al host  │
                   │ Landlock+seccomp      │     └──────────────────────────────────────────┘
                   └───────────────────────┘
```

### 7.2 Flujo de un tick (reflejo) y de una decisión

1. `body.snapshot()` → `Percept` → registro.
2. retina (en `edid`): frame → rejilla de omatidios → tasas → iceoryx2 → `edi-brain`.
3. `edi-brain` avanza Δt biológico → tasas de descendentes → iceoryx2 → motor → `Proposal`.
4. `cortex` (Laya) recibe el `Proposal` junto con las opciones del árbol a11y → `LayaVerdict{p}`.
5. `caps` decide: fantasma (dibujar), pedir permiso o ejecutar. `body.act()` → verificación → `Outcome`.
6. `memory` codifica el episodio. `voice` (a ~1 Hz) escribe la nota. `homeo` ajusta el ritmo.
7. En el sueño, `learn` consume el registro y propone una generación nueva de adaptadores → puerta → A/B.

### 7.3 Workspace Cargo (propuesta)

`edi-schema` (tipos + IDL + stubs Python) · `edid` (binario) · `edi-body` (X11Neko, Niri) · `edi-lif` (motor + validador contra Brian2) · `edi-cortex` · `edi-voice` · `edi-memory` · `edi-learn` · `edi-homeo` · `edi-log` · `edictl` (CLI D-Bus) · `edi-dash` (servidor web del dashboard). Un **solo** `edi.toml` tipado con `serde` y validado en el arranque.

### 7.4 Qué corre dónde

| Pieza | Nube (4 CPU / 15 GB, sin GPU) | VM | USB portátil (RTX 8 GB) |
|---|---|---|---|
| edid | ✅ modo contenedor | ✅ systemd | ✅ systemd |
| Cerebro | subcircuito o completo por debajo del tiempo real; Brian2 → edi-lif | idem | completo; GPU opcional |
| Laya | CPU int8, lotes, ≤ 1 Hz | CPU | GPU (ort CUDA) |
| RWKV-7 0.1B | candle CPU | CPU | web-rwkv (Vulkan) |
| Cuerpo | **neko X11** | sesión Niri | sesión Niri |
| Aprendizaje | EGGROLL pequeño en el sueño | idem | más población; solo con el portátil ocioso y frío |
| Compositor Niri | ⚠️ experimental (headless) | ✅ | ✅ |

---

## 8. Plan por fases (redefine las fases 3–11 de `goal.md` alrededor de `edid`)

| Fase | Entregable | Hecho cuando | Dónde |
|---|---|---|---|
| **E0 · Columna vertebral** | `edi-schema`, `edid` con clock + log (redb) + supervisor en modo contenedor + API zbus + `edictl` | `edictl status` muestra el tick y el registro se reproduce con hash idéntico | nube |
| **E1 · Cuerpo X11** | neko levantado (dockerd) + `X11NekoBody` (atspi + XTest + captura) | `snapshot()` devuelve árbol + frame; acción verificada en xterm/gedit | nube |
| **E2 · Cerebro conectado** | `edi-brain` Brian2 con iceoryx2; retina/motor; test 1 (optomotor) en fantasma | Curva vs. conectoma barajado | nube |
| **E3 · edi-lif** | Motor LIF Rust validado spike a spike contra Brian2 (subred de 800 neuronas + estadística de 30 ensayos, como mlx-lif) | Paridad + **benchmark CPU publicado** | nube |
| **E4 · Córtex y voz** | Laya (ort) y RWKV (candle) en proceso; escalera de capacidades | Laya vs. azar/barajado; notas evaluadas | nube |
| **E5 · Vivo** | homeo (PSI), circadiano, sueño N1–REM, snapshots, A/B de pesos, watchdog | 72 h seguidas sin intervención; reinicio forzado → restaura; una generación mala → rollback automático | nube |
| **E6 · Aprender** | MEM-EGGROLL con LOO-ROLL + canarios + penalización tipo EWC | Curvas vs. ES sin memoria y vs. barajado | nube |
| **E7 · Paquetes** | PKGBUILDs + repo pacman `edios`; Okimarchy empaquetado | `pacman -S edios` en un Arch limpio levanta todo | nube (contenedor Arch) |
| **E8 · Cuerpo Niri** | `NiriBody` (IPC + screencopy + virtual ptr/kbd + atspi con offset de ventana) | Mismos tests que E1 con Niri | VM |
| **E9 · ISO** | fork de omarchy-iso + cidata autoinstall | Arranca e instala en VM con KVM | VM |
| **E10 · USB** | ISO en el portátil; perfil GPU; freno térmico | Arranca; homeo respeta los límites | portátil |

### 8.5 MVP "vivo" en la nube (E0 + E1 + E2 + E5 recortados)

Qué basta para decir honestamente "está vivo" en este contenedor:
1. `edid` corre **siempre** (supervisor propio, reinicio de hijos, heartbeat).
2. Un **cerebro** (subcircuito de Shiu en Brian2, luego `edi-lif`) recibe frames de **neko** y propone movimientos **en fantasma**.
3. **Interocepción** con `/proc/pressure/*`: si la presión sube, el ritmo baja (registrado).
4. **Sueño** programado: compacta el registro, repite episodios y hace **un** paso de ES sobre las ganancias del cerebro con puerta de promoción.
5. **Persistencia**: se mata el contenedor → al volver, restaura snapshot + registro y se ve la continuidad (mismo "yo": hash de linaje).
6. **Dashboard** mínimo: tick, fase vigilia/sueño, PSI, spikes y última acción fantasma.
Sin Niri, sin ISO, sin Laya entrenable. Todo lo anterior es realizable con lo verificado en este documento.

---

## 9. Banderas rojas (honestidad)

| Tema | Riesgo | Mitigación |
|---|---|---|
| Niri en la nube | Headless es de test; salidas virtuales en PR abierta; a11y pide EGL con hardware | Nube = neko X11; Niri desde la VM |
| `ext-image-copy-capture` y captura por ventana en Niri | No está upstream (26.04) | wlr-screencopy + recortar por geometría de la IPC |
| Coordenadas AT-SPI en Wayland | Relativas a la ventana | Sumar la geometría de la ventana (IPC) |
| Cerebro en tiempo real en CPU | Sin benchmark Rust CPU; Brian2 ~8 s por s biológico | Medir en E3; subcircuitos; reloj honesto |
| Laya en CPU | Latencia desconocida (421M) | int8 + lotes + ≤ 1 Hz; GPU en el portátil |
| Okimarchy en ISO | Se instala por script; Omarchy 3 va por ISO + paquetes | Empaquetar (E7) |
| Docker/archiso/QEMU aquí | Daemon parado; privilegios no verificados; sin KVM | Construir el ISO en CI o en una VM con KVM |
| Aprendizaje continuo | SEAL y LoRA olvidan; MEM-EGGROLL es hipótesis nueva | Canarios, A/B de pesos, controles barajados |
| Seguridad (XPIA) | Texto de la UI puede manipular al agente | Laya como juez + capacidades + fantasma por defecto |
| Licencias | FlyWire v783 CC BY-NC 4.0; MaleCNS CC BY 4.0 | Solo uso no comercial con FlyWire |

---

## 10. Fuentes (URLs consultadas)

**Agentes y S.O. de IA**
- AIOS: https://arxiv.org/abs/2403.16971 (html v5: https://arxiv.org/html/2403.16971v5) · repo https://github.com/agiresearch/AIOS · issue scheduling https://github.com/agiresearch/AIOS/issues/554
- MemGPT: https://arxiv.org/pdf/2310.08560 · Letta sleep-time: https://www.letta.com/blog/sleep-time-compute/ · https://www.letta.com/blog/agent-memory/
- Karpathy LLM OS: https://www.frenxt.com/cables/claude-code/karpathy-02-llm-os · https://campedersen.com/llm-os
- OS-Copilot/FRIDAY: https://arxiv.org/abs/2402.07456 · https://github.com/OS-Copilot/OS-Copilot
- UFO²: https://arxiv.org/abs/2504.14603 (html https://arxiv.org/html/2504.14603v1) · https://microsoft.github.io/UFO/ · https://www.microsoft.com/en-us/research/publication/ufo2-the-desktop-agentos/
- Windows agentic: https://petri.com/windows-11-agentic-computing-workspaces/ · https://support.microsoft.com/en-us/windows/experimental-agentic-features-a25ede8a-e4c2-4841-85a8-44839191dfb3 · https://www.windowscentral.com/microsoft/windows-11/microsoft-warns-security-risks-agentic-os-windows-11-xpia-malware
- ANOLISA: https://www.alibabacloud.com/help/en/alinux/agentic-os · https://www.alibabacloud.com/help/en/alinux/alibaba-cloud-linux-4-agentic-edition/
- SUSE MCP: https://www.sdxcentral.com/news/suse-debuts-ai-agents-in-linux-os/ · https://devops.com/suse-extends-ai-agent-reach-via-mcp-server-integration/ · Fedora/Ubuntu: https://www.theregister.com/oses/2026/05/10/both-fedora-and-ubuntu-will-get-ai-support-soon/5237409 · https://www.osnews.com/story/144006/using-ai-to-manage-your-fedora-system-seems-like-a-really-bad-idea/
- Agent S3: https://www.simular.ai/articles/agent-s3 · https://arxiv.org/abs/2510.02250v1 · https://github.com/simular-ai/Agent-S
- UI-TARS-2: https://arxiv.org/abs/2509.02544
- Rabbit/Humane: https://www.engadget.com/rabbit-r1-review-a-199-ai-toy-that-fails-at-almost-everything-161043050.html · https://www.techradar.com/computing/artificial-intelligence/with-the-humane-ai-pin-now-dead-what-does-the-rabbit-r1-need-to-do-to-survive · https://www.digitalapplied.com/blog/ai-product-failures-2026-sora-humane-rabbit-lessons
- 01 Light: https://changes.openinterpreter.com/log/01-app · https://github.com/openinterpreter/01

**OSWorld**
- https://benchlm.ai/benchmarks/osworld-verified · https://leaderboard.steel.dev/leaderboards/osworld/ · https://leaderboard.steel.dev/leaderboards/osworld-2/
- a11y vs. captura: https://proceedings.neurips.cc/paper_files/paper/2024/file/5d413e48f84dc61244b6be550f1cd8f5-Paper-Datasets_and_Benchmarks_Track.pdf · https://arxiv.org/pdf/2506.14866

**Percepción/acción Linux**
- Niri: https://github.com/niri-wm/niri · a11y https://niri-wm.github.io/niri/Accessibility.html · 25.02 virtual-pointer https://linuxiac.com/niri-25-02-wayland-compositor-released/ · 0.1.8 screencopy https://www.phoronix.com/news/Niri-0.1.8-Wayland-Compositor · ext-image-copy issue https://github.com/niri-wm/niri/issues/1558 · PR https://github.com/niri-wm/niri/pull/4554 · PR https://github.com/niri-wm/niri/pull/4599 · salidas virtuales PR https://github.com/niri-wm/niri/pull/3800 · https://github.com/niri-wm/niri/discussions/3101
- Prueba real niri + agente: https://github.com/trycua/cua/issues/3735 · escalado: https://github.com/trycua/cua/issues/3061
- AT-SPI: https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/ · coordenadas Wayland https://github.com/isac322/kwin-mcp/issues/51 · nueva arquitectura GNOME https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/new-protocol.html · https://jocheojeda.com/2026/08/22/at-spi-first-grounding/ · https://github.com/liufeicc/cc-computer-use · crate atspi https://github.com/odilia-app/atspi · https://docs.rs/atspi/latest/atspi/
- ext-image-copy-capture: https://wayland.app/protocols/ext-image-copy-capture-v1
- libei: https://www.phoronix.com/news/libei-1.0-Emulated-Input · https://lists.freedesktop.org/archives/wayland-devel/2020-July/041556.html
- neko: https://github.com/m1k1o/neko · https://neko.m1k1o.net/docs/v3/installation/docker-images
- Sway headless: https://github.com/any1/wayvnc/blob/master/FAQ.md · https://github.com/bbusse/swayvnc

**Arquitectura / IPC / seguridad / estado**
- iceoryx2: https://github.com/eclipse-iceoryx/iceoryx2 · https://iceoryx.io/ · Python v0.7 https://ekxide.io/blog/iceoryx2-0-7-release/
- zbus: https://docs.rs/zbus/latest/zbus/ · https://github.com/z-galaxy/zbus
- Cap'n Proto Rust: https://github.com/capnproto/capnproto-rust · https://docs.rs/capnp-rpc/
- PyO3: https://pyo3.rs/main/free-threading · https://pyo3.rs/main/building-and-distribution
- Landlock: https://docs.kernel.org/userspace-api/landlock.html · https://github.com/landlock-lsm/rust-landlock
- bubblewrap: https://github.com/containers/bubblewrap · https://wiki.archlinux.org/title/Bubblewrap
- CRIU: https://criu.org/Main_Page · https://access.redhat.com/articles/2455211 · https://oneuptime.com/blog/post/2026-01-16-docker-checkpoint-restore/view
- redb: https://github.com/cberner/redb

**Cerebro / modelos**
- Shiu LIF: https://pmc.ncbi.nlm.nih.gov/articles/PMC10187186/ · https://github.com/philshiu/Drosophila_brain_model
- mlx-lif: https://github.com/Kisame76/mlx-lif-engine · https://github.com/Kisame76/drosophila-brain-mlx
- flyBrain Rust+Metal: https://github.com/mehrantsi/flyBrain · fly-brain multi-backend: https://github.com/eonsystemspbc/fly-brain · lista: https://github.com/cobanov/awesome-fly
- Crates SNN: https://docs.rs/oxicuda-snn/latest/oxicuda_snn/ · https://github.com/Limen-Neural/neuromod · https://docs.rs/neuralos-snn/latest/neuralos_snn/ · https://github.com/marcopra/SpikingNN_Rust · https://docs.rs/oldies-brian/latest/oldies_brian/
- Laya: https://huggingface.co/convaiinnovations/laya · https://laya.convaiinnovations.com/
- candle: https://github.com/huggingface/candle · modelos https://docs.rs/candle-transformers/latest/candle_transformers/models/index.html · RWKV ejemplo https://github.com/huggingface/candle/tree/main/candle-examples/examples/rwkv
- ort: https://docs.rs/ort · https://docs.rs/crate/ort/latest
- RWKV Rust/CPU: https://github.com/cryscan/web-rwkv · https://github.com/RWKV/rwkv.cpp · https://github.com/ggml-org/llama.cpp/pull/12412 · https://github.com/cgisky1980/rwkv-rsv

**Proceso vivo**
- PSI: https://docs.kernel.org/accounting/psi.html · https://facebookmicrosites.github.io/cgroup2/docs/pressure-metrics.html · systemd-oomd https://man.archlinux.org/man/systemd-oomd.8 · triggers sin privilegios https://lkml.iu.edu/hypermail/linux/kernel/2303.2/10470.html
- Watchdog: https://www.freedesktop.org/software/systemd/man/latest/sd_notify.html · https://github.com/iddm/sdwd
- Boot counting: https://systemd.io/AUTOMATIC_BOOT_ASSESSMENT/ · https://www.freedesktop.org/software/systemd/man/latest/systemd-bless-boot.service.html
- Sueño: https://www.nature.com/articles/s41467-022-34938-7

**Imagen de S.O.**
- omarchy-iso: https://github.com/omacom/omarchy-iso · manual https://learn.omacom.io/2/the-omarchy-manual/96/manual-installation
- Okimarchy: https://github.com/cristian-fleischer/okimarchy · https://github.com/baodon2703/okimarchy
- mkosi: https://wiki.archlinux.org/title/Mkosi · https://github.com/Amphero/arch-image-based · UKI https://wiki.archlinux.org/title/Unified_kernel_image
- archiso en Docker: https://bbs.archlinux.org/viewtopic.php?id=288102 · https://github.com/nlhomme/archiso-builder
- QEMU MTTCG: https://www.qemu.org/docs/master/devel/multi-thread-tcg.html · https://www.linaro.org/blog/qemu-a-tale-of-performance-analysis/

**Aprendizaje continuo**
- EGGROLL: https://arxiv.org/abs/2511.16652 · https://eshyperscale.github.io/ · LOO-ROLL: https://arxiv.org/abs/2609.10980
- SEAL: https://arxiv.org/pdf/2506.10943 · https://github.com/Continual-Intelligence/SEAL
- EWC: https://pubmed.ncbi.nlm.nih.gov/28292907/
- LoRA y olvido: https://zylos.ai/research/2026-04-09-continual-learning-catastrophic-forgetting-ai-agents/ · https://iclr.cc/virtual/2026/poster/10008003
- Nested Learning: https://research.google/blog/introducing-nested-learning-a-new-ml-paradigm-for-continual-learning/
- TTT-E2E: https://bdtechtalks.com/2026/01/12/nvidia-end-to-end-test-time-training/ · workshop https://ttcl-agents.github.io/
