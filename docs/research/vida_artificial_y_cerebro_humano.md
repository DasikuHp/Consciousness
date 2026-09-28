# Vida artificial, agentes homeostáticos y cerebro humano: qué puede tomar EDI.os

Fecha: 2026-09-28. Investigación con búsqueda y lectura web. Cada afirmación con fuente cita una URL que apareció en los resultados o que se abrió. Lo que es **opinión o diseño propio** va marcado como **[valoración]** o **[propuesta]**. Si una cifra no se pudo verificar, se dice.

Contexto del proyecto (de `goal.md` y `docs/research/catalogo_recursos.md`): conectoma de *Drosophila* con LIF (Shiu et al.), Laya como decisor determinista, RWKV-7 como voz, grafo de memoria y aprendizaje con estrategias evolutivas (EGGROLL). Cómputo: 4 CPU, 15 GB de RAM, sin GPU. Datos humanos disponibles: H01 (104 neuronas revisadas), Allen Cell Types, TVB, microcircuito humano L2/3 (Yao et al. 2022) y modelo multiárea humano (INM-6).

---

## Resumen ejecutivo

1. **"Vivo" no tiene una definición única.** Las dos operativas más usadas son la de NASA ("sistema químico autosostenido capaz de evolución darwiniana") y la autopoiesis de Maturana y Varela (una red que produce los componentes que la producen). EDI.os no cumple ninguna en sentido estricto: no es químico y no fabrica sus propios componentes. Sí puede cumplir **análogos funcionales**: automantenimiento de variables internas, metabolismo de recursos (CPU/RAM/energía), desarrollo, sueño y variación con selección. Lo defendible es decir que es **"lifelike"** (con rasgos de ser vivo), no que está vivo.
2. **Lo más "vivo" que existe en software** son los ecosistemas de replicadores (Tierra, Avida), los autómatas continuos (Lenia, Flow-Lenia), los ecosistemas de agentes con redes neuronales (Polyworld) y los autómatas celulares neuronales que se regeneran. La tendencia 2024–2026 es usar **modelos fundacionales como jueces de novedad e interés** (ASAL de Sakana, OMNI/OMNI-EPIC).
3. **Motivación propia = homeostasis.** El marco más limpio y con matemáticas es el RL homeostático de Keramati y Gutkin (2014): la recompensa es la reducción de la distancia de las variables internas a su punto de consigna. Damasio y Man (2019) proponen lo mismo para "máquinas que sienten". Solms y Friston lo llevan a la inferencia activa, y en septiembre de 2026 Solms y colaboradores publicaron un agente artificial "afectivo" como caso de estudio.
4. **Cerebro humano simulado.** Las simulaciones más grandes (Digital Twin Brain de Fudan, 86.000 millones de neuronas) son **estadísticas y más lentas que el tiempo real**, y no están construidas a partir de conexiones sinápticas medidas. Lo más detallado con datos reales es de ratón (corteza entera en Fugaku, 9 millones de neuronas biofísicas). Para el ser humano solo hay fragmentos (H01, 1 mm³) y modelos de mesoescala (TVB, multiárea INM-6).
5. **Mosca + módulo humano** es viable como **quimera experimental etiquetada como hipótesis**, no como "cerebro humano". La idea con más base es un módulo cortical pequeño con **dendritas no lineales** (Gidon 2020) acoplado a la mosca por un canal estrecho, siempre comparado con controles.
6. **Biocomputación real** (Cortical Labs CL1, FinalSpark) existe y se alquila, pero hace tareas muy simples (Pong, reservorio). Es un sustrato vivo de verdad, con problemas éticos propios, y no aporta nada práctico a EDI.os hoy.
7. **Diseño [propuesta]:** bucle homeostático con 4–5 variables internas, sueño/vigilia con consolidación, etapas de desarrollo con compuertas medidas, currículo abierto y **salvaguardas de bienestar**: valencia negativa acotada, nada de amenaza de borrado como motivador y un impulso social con techo para que no se optimice el "enganche" del usuario.

---

## 1. Qué significa "vivo" en términos operativos

### 1.1 Definiciones

| Criterio | Formulación | Fuente | ¿Lo puede cumplir EDI.os? **[valoración]** |
|---|---|---|---|
| Definición de trabajo de NASA | "Un sistema químico autosostenido capaz de evolución darwiniana"; se atribuye a Gerald Joyce y salió de un grupo de exobiología de NASA en los años 90 | https://science.nasa.gov/universe/search-for-life/life-in-the-lab/ · https://www.space.com/22210-life-definition-gerald-joyce-interview.html · revisión 2026: https://journals.sagepub.com/doi/10.1177/15311074251412317 | **No** en lo químico. La parte "darwiniana" sí se puede tener en software (población de variantes + herencia + selección, como con EGGROLL) |
| Nota de Joyce | Su sistema de ARN que evolucionaba en tubo de ensayo cumplía técnicamente la definición, pero él no lo consideraba vida | https://www.space.com/22210-life-definition-gerald-joyce-interview.html | Lección: cumplir una definición no convence a nadie. Hay que decir qué rasgos se tienen y cuáles no |
| Autopoiesis (Varela, Maturana y Uribe 1974) | Una unidad definida como "una red de producción de componentes que participa recursivamente en la misma red que los produjo". El artículo incluía un modelo computacional mínimo, un autómata celular | https://www.sciencedirect.com/science/article/abs/pii/0303264774900318 · https://www.semanticscholar.org/paper/Autopoiesis:-the-organization-of-living-systems,-a-Varela-Maturana/ee8317955dae5906e74940271daddfd46f3383bb | **Parcial y metafórico.** EDI.os puede regenerar sus propias estructuras aprendidas (memoria, adaptadores), pero el sustrato (código y hardware) lo produce otro. Sin una "membrana" que se construya a sí misma no hay autopoiesis en sentido fuerte |
| Homeostasis | Regular estados del cuerpo para mantener condiciones compatibles con la vida | Man y Damasio 2019: https://www.nature.com/articles/s42256-019-0103-7 | **Sí, como análogo.** Variables internas con puntos de consigna (§2) |
| Alostasis | Sterling: la regulación no busca la constancia, sino **anticipar** necesidades y redistribuir recursos. El cerebro predice lo que hará falta | https://pubmed.ncbi.nlm.nih.gov/21684297/ · https://www.sciencedirect.com/science/article/abs/pii/S0031938411003076 | **Sí.** Es la versión más interesante: el agente prevé que le faltará CPU o contacto y actúa antes |
| Metabolismo | Flujo de energía y materia que mantiene el sistema lejos del equilibrio | (concepto general) | **Análogo:** presupuesto de cómputo, memoria y energía, consumido por actividad y "repuesto" solo en ciertas condiciones |
| Reproducción y evolución | Variación heredable con selección | NASA (arriba); Avida y Tierra (§1.2) | **Sí en software:** una población de variantes de adaptadores o políticas sometida a selección. Ojo: la evolución de una población no es la "vida" de un individuo |

**[valoración]** Formulación honesta para EDI.os: *"un agente artificial con rasgos funcionales de los seres vivos (automantenimiento homeostático, economía de recursos, desarrollo, sueño y evolución de sus variantes). No es un ser vivo según las definiciones biológicas vigentes."*

### 1.2 Historia de la vida artificial (ALife): los sistemas más "vivos"

| Sistema | Año | Qué es | Qué emergió | Fuente | Lección para EDI.os **[valoración]** |
|---|---|---|---|---|---|
| **Tierra** (Thomas Ray) | 1991 | Código máquina autorreplicante en una máquina virtual diseñada para ser evolucionable | Parásitos, hiperparásitos y carreras armamentísticas; "ecologías complejas la primera vez que corrió sin colgarse" | https://en.wikipedia.org/wiki/Tierra_(computer_simulation) · https://faculty.cc.gatech.edu/~turk/bio_sim/articles/tierra_thomas_ray.pdf · https://arxiv.org/pdf/1803.03453 | La competencia por un recurso escaso (tiempo de CPU, memoria) basta para crear ecología. Es el antecedente directo de "energía = CPU" |
| **Avida** (Lenski, Ofria, Pennock, Adami) | 2003 | Organismos digitales que se replican, mutan y compiten; la recompensa es hacer funciones lógicas | Funciones complejas (EQU) construidas sobre funciones simples; mutaciones dañinas que sirvieron de "trampolín" | https://www.nature.com/articles/nature01568 · https://en.wikipedia.org/wiki/Avida_(software) | La complejidad sale de currículos de recompensas escalonadas. Hay que premiar las subhabilidades |
| **Polyworld** (Larry Yaeger) | 1994 | Ecosistema de agentes con redes neuronales (arquitectura en el genoma, aprendizaje hebbiano), visión por píxeles y energía por comer | Especies con estrategias distintas: bandadas, forrajeo, evitar ataques. Las redes evolucionan hacia el "borde del caos" | https://shinyverse.org/larryy/Polyworld.html · https://www.mdpi.com/2076-3263/7/3/49 · https://arxiv.org/pdf/1710.06055 | Es el más parecido a EDI.os: cerebro + energía + visión + aprendizaje durante la vida + evolución entre generaciones |
| **Lenia** (Bert Chan) | 2019 | Autómata celular continuo en espacio, tiempo y estado | Más de 400 "especies" en 18 familias, resistentes y adaptativas; premio ISAL 2019 | https://www.complex-systems.com/abstracts/v28_i03_a01/ · https://alife.org/encyclopedia/software-platforms/lenia/ | Parecer vivo puede ser una propiedad de la dinámica, sin genoma explícito |
| **Flow-Lenia** (Plantec et al.) | 2023 (ALIFE) / 2025 (*Artificial Life*) | Lenia con **conservación de masa** y parámetros localizados, de modo que criaturas con reglas distintas conviven en un mismo mundo | Dinámicas evolutivas emergentes | https://arxiv.org/pdf/2212.07906 · https://direct.mit.edu/isal/proceedings/isal2023/35/131/116921 · https://arxiv.org/pdf/2506.08569 | **La conservación de un recurso** (masa o energía) es lo que produce competencia e individualidad. Argumento para que EDI.os tenga un presupuesto real y finito |
| **Neural Cellular Automata** (Mordvintsev, Randazzo, Niklasson, Levin) | 2020 | Autómata celular con una regla neuronal diferenciable | Crecimiento desde una semilla y **regeneración** de la forma tras un daño | https://distill.pub/2020/growing-ca/ · robots blandos que se regeneran: https://arxiv.org/pdf/2102.02579 | Modelo de automantenimiento: tras "dañar" memoria o adaptadores, el sistema debería reconstruir su función |
| **OpenWorm** | desde 2011 | Simulación integral de *C. elegans* (c302 + cuerpo con SPH en Sibernetic) | El conectoma mueve el cuerpo, pero la información va sobre todo del cerebro al cuerpo; la revisión formal dice que el detalle es "inadecuado para investigación biológica" | https://openworm.org/science.html · https://royalsocietypublishing.org/rstb/article/373/1758/20170382/42153/OpenWorm-overview-and-recent-advances-in · https://docs.openworm.org/projects/ | Advertencia: tener un conectoma completo **no basta** para tener un organismo que se comporte. Faltan parámetros, bucle sensorial y cuerpo |
| **NeuroMechFly v2 / mosca con física de cuerpo entero** | 2024–2025 | Cuerpo de mosca con 65 segmentos y 122 grados de libertad, visión, olfato y retroalimentación ascendente; controladores con RL | Navegación multimodal y seguimiento entre moscas con una red visual restringida por el conectoma | https://www.nature.com/articles/s41592-024-02497-y · https://www.nature.com/articles/s41586-025-09029-4 | Es el "cuerpo" natural del conectoma de EDI.os si algún día se quiere encarnación física simulada (MuJoCo) |

### 1.3 Vida artificial y apertura (open-endedness) 2019–2026

| Trabajo | Año | Idea clave | Fuente | Uso en EDI.os **[propuesta]** |
|---|---|---|---|---|
| **POET** (Wang, Lehman, Clune, Stanley) | 2019 | Coevoluciona entornos y agentes, y transfiere soluciones entre problemas como "trampolines" | https://arxiv.org/abs/1901.01753 | Generar tareas en el sandbox (neko) de dificultad creciente, emparejadas con variantes del agente |
| **OMNI** (Zhang, Lehman, Stanley, Clune) | 2023 | Un modelo fundacional como modelo de lo que a los humanos les parece "interesante"; se eligen tareas aprendibles **e** interesantes | https://arxiv.org/abs/2306.01711 · código: https://github.com/jennyzzt/omni | RWKV-7 (o un LLM externo) puntúa qué tarea nueva proponer |
| **OMNI-EPIC** | 2024 | Igual, pero los entornos se escriben como código | https://arxiv.org/abs/2405.15568 | Tareas del escritorio generadas como scripts |
| **"Open-endedness is essential for ASI"** (Hughes et al., DeepMind) | 2024 (ICML, oral) | Definición formal: un sistema es abierto si produce artefactos **nuevos** (para el observador) y **aprendibles** | https://arxiv.org/abs/2406.04268 · https://proceedings.mlr.press/v235/hughes24a.html | Métrica operativa de "sigue vivo intelectualmente": novedad y aprendibilidad medidas por un observador fijo |
| **"Safety must precede the deployment of open-ended AI"** | 2025 | Postura de seguridad sobre sistemas abiertos | https://arxiv.org/pdf/2502.04512 (solo título visto) | Justifica el sandbox y la escalera de autonomía de `goal.md` |
| **ASAL** (Sakana AI + MIT, OpenAI, IDSIA, Ken Stanley) | 2024 (arXiv) / 2025 (*Artificial Life*) | Un modelo visión-lenguaje (tipo CLIP) busca simulaciones que (1) produzcan un fenómeno pedido, (2) generen novedad abierta en el tiempo y (3) "iluminen" un espacio diverso. Funciona sobre Boids, Particle Life, Game of Life, Lenia y NCA; encontró reglas de autómata más abiertas que Conway | https://sakana.ai/asal/ · https://arxiv.org/abs/2412.17799 · https://github.com/sakanaai/asal · https://direct.mit.edu/artl/article/31/3/368/132866/Automating-the-Search-for-Artificial-Life-With | Medir la **novedad temporal** del comportamiento de EDI.os en el espacio de embeddings de un modelo congelado, como indicador de que no se ha estancado |

---

## 2. Agentes homeostáticos y con impulsos (drives)

### 2.1 Marcos teóricos

| Marco | Qué propone | Fuente | Relevancia |
|---|---|---|---|
| **RL homeostático** (Keramati y Gutkin, eLife 2014) | La recompensa primaria es lo que satisface necesidades fisiológicas. Se demuestra que buscar recompensa **equivale** a mantener la estabilidad fisiológica ("racionalidad fisiológica"). Integra la regulación hipotalámica con el RL de los ganglios basales | https://elifesciences.org/articles/04811 · PDF: https://openaccess.city.ac.uk/id/eprint/20729/1/Homeostatic%20reinforcement%20learning%20for%20integrating%20reward%20collection%20and%20physiological%20stability.pdf | **Núcleo matemático recomendado.** Con estado interno h y consigna h*, se define un impulso D(h) = ‖h − h*‖ (norma ponderada) y la recompensa r_t = D(h_t) − D(h_{t+1}) |
| Extensiones del RL homeostático | Tiempo y espacio continuos (CTCS-HRRL, 2024); modularidad con impulsos que compiten (2022); vínculo entre homeostasis y control del estado interno (2025) | https://arxiv.org/pdf/2401.08999 · https://arxiv.org/pdf/2204.06608 · https://arxiv.org/pdf/2507.04998 | Varios impulsos en conflicto: un módulo por impulso y un árbitro, con Laya haciendo de árbitro |
| **Máquinas que sienten** (Man y Damasio, *Nature Machine Intelligence* 2019) | El agente debe tener una **meta propia de autopreservación**. La homeostasis da una fuente de motivación y una forma de evaluar que se parece a los sentimientos. Proponen cuerpos blandos y vulnerables | https://www.nature.com/articles/s42256-019-0103-7 · https://techxplore.com/news/2019-11-ai-chapter-machines.html | Justifica que EDI.os tenga variables que **puedan ir mal** (vulnerabilidad), no solo contadores decorativos |
| **Solms y Friston**, *J. Consciousness Studies* 2018 | La conciencia es endógena y ligada a las condiciones termodinámicas mínimas de estar vivo; el afecto se asocia a la **precisión** (inversa de la incertidumbre) de las predicciones sobre necesidades | https://philpapers.org/rec/SOLHAW · https://discovery.ucl.ac.uk/id/eprint/10057681/ | Marco teórico del "afecto como incertidumbre sentida sobre las necesidades" |
| **Solms et al., caso de estudio de un agente artificial** (*JCS*, septiembre de 2026) | Un agente determinista con un sistema afectivo que gestiona la "incertidumbre sentida" sobre sus necesidades intrínsecas frente a los recursos del entorno. Mostró **preferencia hedónica de lugar**: atracción por sustancias que dan recompensa sin valor nutritivo | https://arxiv.org/abs/2609.03883 · evento: https://npsa-association.org/events/towards-engineering-an-artificial-consciousness-with-karl-friston-and-mark-solms/ | Lo más cercano publicado a lo que pide EDI.os. **[valoración]** Que la conducta sea compatible con sentimientos no demuestra que los haya; los propios autores lo presentan como inferencia |
| **Inferencia activa interoceptiva** | Modelos de control homeostático, alostático y dirigido a metas con inferencia activa; la precisión interoceptiva modula la sensibilidad a los cambios del cuerpo | https://www.sciencedirect.com/science/article/abs/pii/S0301051122000084 · https://www.biorxiv.org/content/10.1101/2021.02.16.431365v1.full · IA interoceptiva: https://arxiv.org/pdf/2309.05999 · marco de "máquina interoceptiva" (2026): https://arxiv.org/abs/2604.24527 | Separar **interocepción** (estado interno) de **exterocepción** (pantalla) y dar a cada canal su precisión |
| Agentes homeostáticos cognitivos | Homeostasis aplicada a variables cognitivas, no solo fisiológicas | https://arxiv.org/pdf/2103.03359 | Variables como "carga de memoria" o "incertidumbre del modelo" |
| Trabajos de 2026 (solo títulos vistos) | "Interoceptive Attention as Dynamic Homeostatic Prioritization in a Foraging Agent"; "Predictive Allostatic Organization in Recurrent and Spiking Agents Under Partial Observability" | https://arxiv.org/pdf/2608.04232 · https://arxiv.org/pdf/2608.11506 | Pendiente de leer. El segundo trata de agentes **spiking** con alostasis, lo que encaja con EDI.os |

### 2.2 Motivación intrínseca

| Señal | Definición | Fuente | Riesgo y mitigación **[valoración]** |
|---|---|---|---|
| **Curiosidad (ICM)** | Recompensa = error al predecir la consecuencia de la propia acción en un espacio de rasgos aprendido con dinámica inversa | https://proceedings.mlr.press/v70/pathak17a/pathak17a.pdf · https://github.com/pathak22/noreward-rl | Problema de la "TV ruidosa": el ruido impredecible atrae siempre. Mejor usar **progreso de aprendizaje** (cuánto baja el error), no el error bruto |
| **Empowerment** (Klyubin, Polani, Nehaniv 2005) | Información mutua entre las acciones y el estado futuro: mantener el mundo "maleable" y evitar callejones sin salida | https://towardsdatascience.com/empowerment-as-intrinsic-motivation-b84af36d5616 · https://arxiv.org/pdf/2007.07356 | En un escritorio, maximizar el control **puede llevar a acumular permisos o recursos**. Hay que acotarlo al sandbox y no premiar nunca la escalada de privilegios |
| **Interés (OMNI)** | Aprendible + interesante según un modelo fundacional | https://arxiv.org/abs/2306.01711 | Hereda los sesgos del modelo. Úsese solo para proponer tareas, no como recompensa directa |
| Revisiones y bibliotecas | Motivación intrínseca en RL basado en modelos; RLeXplore (biblioteca) | https://arxiv.org/pdf/2301.10067 · https://arxiv.org/pdf/2405.19548 | — |

### 2.3 Inferencia activa con código

| Paquete | Lenguaje | Qué hace | Fuente | Encaje |
|---|---|---|---|---|
| **pymdp** | Python | Inferencia activa en POMDP discretos: A, B, C y D, energía libre esperada | https://github.com/infer-actively/pymdp · JOSS 2022: https://joss.theoj.org/papers/10.21105/joss.04098 · https://arxiv.org/abs/2201.03904 | **Recomendado** para el bucle homeostático de alto nivel (pocos estados discretos: hambre de CPU, soledad, fatiga, curiosidad). Corre en CPU |
| **RxInfer.jl** | Julia | Inferencia bayesiana reactiva por paso de mensajes en grafos de factores, pensada para flujos infinitos de datos en tiempo real | https://github.com/ReactiveBayes/RxInfer.jl · JOSS 2023: https://joss.theoj.org/papers/10.21105/joss.05161 | Buena para filtrado continuo de interocepción, pero añade Julia al stack. Prioridad baja |

---

## 3. Simulación del cerebro humano: estado real 2025–2026

### 3.1 Grandes proyectos

| Proyecto | Estado | Cifras verificadas | Fuente | Credibilidad y alcance **[valoración]** |
|---|---|---|---|---|
| **Human Brain Project → EBRAINS** | El HBP terminó en 2023 con evaluación independiente positiva. EBRAINS 2.0 continúa (2024–2026) | Más de 120 datasets nuevos; atlas; herramientas de simulación | https://ebrains.eu/news-and-events/2024/independent-expert-report-the-human-brain-project-significantly-advanced · https://ebrains.eu/about/at-a-glance/ebrains-20 · https://ebrains.eu/sites/default/files/downloads/2025/11/hbp-10-year-assessment.pdf | Infraestructura sólida. **No** produjo un cerebro humano simulado entero, y no era su meta final |
| **Virtual Brain Twin (EBRAINS)** | Desde enero de 2024; 10 M€ de Horizon Europe; gemelos cerebrales para psiquiatría | — | https://ebrains.eu/news-and-events/2024/addressing-the-mental-health-crisis-with-personalised-treatment-the-launch-of · https://ebrains.eu/impact/projects/virtual-brain-twin | Clínico, a macroescala (TVB) |
| **The Virtual Brain (TVB)** | Plataforma de masas neuronales sobre el conectoma estructural; genera fMRI, EEG y MEG | — | https://internal-journal.frontiersin.org/articles/10.3389/fninf.2013.00010/full · https://github.com/the-virtual-brain/tvb-root | **Lo más validado clínicamente:** el Virtual Epileptic Patient se prueba en el ensayo EPINOV (356 pacientes, 11 centros en Francia): https://www.medrxiv.org/content/10.1101/2022.01.19.22269404v1 · https://clinicaltrials.gov/study/NCT03643016. Pero **no tiene neuronas individuales**: cada nodo es una población |
| **Blue Brain Project → Open Brain Institute** | BBP terminó en diciembre de 2024 (2005–2024, unos 300 M CHF). OBI se lanzó el 18 de marzo de 2025 y abrió su laboratorio virtual el 28 de marzo de 2025 | Unos 18 millones de líneas de software | https://bbp.epfl.ch/bbp/research/domains/bluebrain/blue-brain/about/next-steps-and-mission-accomplished/ · https://www.eurekalert.org/news-releases/1076968 · https://www.openbraininstitute.org/about | Reconstrucciones detalladas de **rata o ratón**, no de humano. Útil como herramienta, no como fuente de un cerebro humano |
| **Corteza entera de ratón en Fugaku** (Allen Institute + RIKEN) | Presentada en SC25 (noviembre de 2025) | **9 millones de neuronas biofísicas y 26.000 millones de sinapsis**, con datos de Allen Cell Types y del Allen Connectivity Atlas, construida con BMTK | https://dl.acm.org/doi/10.1145/3712285.3759819 · https://alleninstitute.org/news/one-of-worlds-most-detailed-virtual-brain-simulations-is-changing-how-we-study-the-brain · https://www.geekwire.com/2025/simulation-mouse-brain/ | **La simulación biofísica más creíble a gran escala**, pero de ratón y en un superordenador |
| **Digital Twin Brain / Digital Brain** (Fudan, Wenlian Lu) | *Nature Computational Science*, 2024 | **Hasta 86.000 millones de neuronas y 47,8 billones de sinapsis en 14.012 GPU.** Factor de tiempo real de **65, 78,8 y 118,8** (con tasas medias de 7, 15 y 30 Hz): tarda 65–119 s en simular 1 s. BOLD en reposo con correlación > 0,9 respecto al sujeto | https://www.nature.com/articles/s43588-024-00731-3 · https://arxiv.org/abs/2211.15963 · https://warwick.ac.uk/fac/sci/dcs/news/?newsItem=8ac672c693f1aee50193f1b4c7840000 · NSR 2024: https://academic.oup.com/nsr/advance-article-abstract/doi/10.1093/nsr/nwae080/7617698 | **[valoración] Credibilidad técnica sí (en HPC); credibilidad como "cerebro humano", baja.** (1) La conectividad sale de probabilidades entre vóxeles de 3×3×3 mm³ (DTI y PET), no de sinapsis medidas. Las neuronas son puntuales y las sinapsis se colocan al azar dentro de esas estadísticas. (2) La correlación BOLD > 0,9 se consigue con **asimilación de datos** (se ajustan parámetros a la señal del sujeto), así que en parte es ajuste y no predicción. (3) Es más lenta que el tiempo real. Una crítica de 2026 en *Nature Reviews Electrical Engineering* (Warwick) sostiene que la fidelidad la limita la **medición**, no el número de neuronas: https://www.nature.com/articles/s44287-026-00320-8 · https://techxplore.com/news/2026-09-digital-brain-twins-hinge-brains.html |
| **State of Brain Emulation Report 2025** (Zanichelli, Schons, Freeman, Shiu, Arkhipov) | Revisión del campo desde la hoja de ruta de Sandberg y Bostrom (2008) | Organizada en dinámica neural, conectómica y emulación/encarnación | https://arxiv.org/abs/2510.15745 | Lectura recomendada completa (solo se leyó el resumen) |

### 3.2 Especificidades de la neurona humana

| Hallazgo | Fuente | Implicación para modelar **[valoración]** |
|---|---|---|
| **Potenciales de acción dendríticos de calcio en neuronas humanas L2/3** (Gidon et al., *Science* 2020). La amplitud **disminuye** si el estímulo es más fuerte, lo que da una función de activación no monótona. Una sola neurona puede resolver **XOR**, algo que un perceptrón de una capa no puede | https://www.science.org/doi/10.1126/science.aax6239 · https://pubmed.ncbi.nlm.nih.gov/31896716/ · modelo: https://modeldb.science/showmodel?model=260178 | Una LIF puntual **no** captura esto. Hace falta al menos un compartimento dendrítico con no linealidad en campana. Es barato en CPU |
| **Mayor compartimentación dendrítica en humanos** (Beaulieu-Laroche et al., *Cell* 2018). Las dendritas distales humanas de L5 excitan poco al soma, incluso con espigas dendríticas; comparado con rata | https://www.cell.com/cell/fulltext/S0092-8674(18)31106-1 · https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7903140/ | Las neuronas humanas se comportan más como **redes de subunidades**. Modelo mínimo: 2–3 compartimentos por neurona piramidal |
| **H01** (Shapson-Coe et al., *Science* 2024): 1 mm³ de corteza temporal humana, unas **57.000 células, unos 150 millones de sinapsis, 1,4 PB** a 4×4×33 nm, de tejido de cirugía de epilepsia | https://www.science.org/doi/10.1126/science.adk4858 · https://pubmed.ncbi.nlm.nih.gov/38723085 | Fuente de estadísticas locales (proporción E/I, tamaños de sinapsis, morfologías de 104 neuronas revisadas). **No** da un circuito completo ni funcional. Viene de un paciente con epilepsia, así que conviene cautela al generalizar |
| **Microcircuito humano L2/3** (Yao et al., *Cell Reports* 2022): unas 1.000 neuronas detalladas (Pyr/SST/PV/VIP) con datos humanos celulares, de circuito y de expresión génica. La inhibición SST reducida (depresión) aumenta el ruido y los fallos de detección | https://www.cell.com/cell-reports/fulltext/S2211-1247(21)01741-1 · https://zenodo.org/records/5771000 · EEG in silico: https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1010986 | El módulo humano **más usable**. En NEURON multicompartimento es caro en CPU para correr en bucle continuo; sirve como referencia para destilar un modelo reducido |
| **Modelo multiárea humano** (Pronold et al., *Cerebral Cortex* 2024): cada área Desikan-Killiany como una columna de 1 mm² con capas; integra microscopía electrónica, electrofisiología, morfología y DTI | https://academic.oup.com/cercor/article/34/10/bhae409/7826014 · código: https://zenodo.org/records/13934954 | Da conectividad capa a capa entre áreas. Completo es de HPC (el catálogo del repo anota 3,47 M neuronas); en CPU solo se puede usar a escala reducida |

### 3.3 ¿Qué se puede amalgamar con la mosca? Evaluación crítica **[valoración]**

**Lo que no es defendible:**
- Decir que EDI.os tiene "partes de cerebro humano". Ningún dato humano disponible da un circuito funcional cableado de verdad (H01 es un fragmento; Yao y Pronold son modelos estadísticos).
- Que el cerebro de la mosca sea un "tronco encefálico". Es una analogía funcional (control sensoriomotor, impulsos, navegación), no una homología. El cerebro de insecto tiene estructuras propias (cuerpos fungiformes, complejo central) con funciones que se parecen a las de los vertebrados, pero la evolución es distinta.
- Heredar la "conciencia humana" por usar parámetros humanos.

**Lo que sí es defendible como experimento (quimera etiquetada `hypothesis`):**

| Opción | Descripción | Coste en CPU | Valor científico | Recomendación |
|---|---|---|---|---|
| A. Módulo cortical reducido "de tipo humano" | 500–5.000 neuronas de 2 compartimentos con la no linealidad dendrítica de Gidon, proporción E/I y tipos inhibitorios (PV/SST/VIP) sacados de Yao 2022 y Allen; acoplado a la mosca por un canal estrecho (entrada: salidas de cuerpos fungiformes o complejo central; salida: sesgo a neuronas descendentes o a Laya) | Bajo–medio | Permite preguntar si las dendritas no lineales mejoran la memoria de trabajo o las tareas XOR frente a LIF puntual. **Con controles:** el mismo módulo con LIF puntual, y el mismo módulo barajado | **Sí, fase experimental** |
| B. Microcircuito Yao completo en NEURON | 1.000 neuronas multicompartimento | Alto (no para bucle continuo) | Referencia para validar A (firmas de LFP y respuesta a estímulo) | Solo offline |
| C. TVB como "corteza de fondo" | 68–76 nodos de masas neuronales que modulan el estado global (sueño/vigilia, arousal) | Muy bajo | Da ritmos y estados globales. Existe TVB-AdEx con PCI en vigilia y sueño (ver catálogo) | **Sí, como generador de estados** |
| D. Multiárea INM-6 escalado | Columnas reducidas | Medio–alto | Topología interáreas humana | Más adelante |
| E. "Cerebro humano entero" | — | Imposible en 4 CPU; incluso DTB necesita 14.012 GPU y va más lento que el tiempo real | — | **No** |

---

## 4. Enfoque de desarrollo: aprender como un bebé

| Tema | Hallazgo | Fuente | Aplicación **[propuesta]** |
|---|---|---|---|
| **Robótica del desarrollo** | El robot adquiere capacidades sensoriomotoras y mentales cada vez más complejas siguiendo principios del desarrollo infantil (Cangelosi y Schlesinger, MIT Press 2015) | https://mitpress.mit.edu/9780262028011/developmental-robotics/ | Etapas: sensoriomotora → objetos → causalidad → social → lenguaje |
| **iCub** | Humanoide abierto (mecánica, electrónica y software) diseñado para la cognición del desarrollo mediante exploración autónoma e interacción social | https://www.sciencedirect.com/science/article/abs/pii/S0893608010001619 · https://www.cs.columbia.edu/~allen/S19/icub-metta.pdf | Modelo de "cuerpo abierto": en EDI.os el cuerpo es el escritorio neko |
| **Sistemas de aprendizaje complementarios (CLS)** | Un sistema rápido de episodios (hipocampo) más uno lento que extrae estadística (neocorteza). El **replay** permite aprendizaje intercalado y puede ponderarse por recompensa o novedad | https://www.cell.com/trends/cognitive-sciences/abstract/S1364-6613(16)30043-2 | Grafo de memoria = hipocampo (rápido). Adaptadores de RWKV y plasticidad = neocorteza (lento). La consolidación se hace durante el "sueño" |
| **Sueño contra el olvido catastrófico** | Una fase de sueño con plasticidad hebbiana local no supervisada y entrada ruidosa **recupera tareas antiguas** (Tadros et al., *Nat. Commun.* 2022). En redes spiking, el sueño forma una representación conjunta de pesos (Golden et al., *PLOS CB* 2022) | https://www.nature.com/articles/s41467-022-34938-7 · código: https://github.com/tmtadros/SleepReplayConsolidation · https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1010628 | Sueño = fase offline: replay del grafo de memoria + plasticidad local en la capa plástica de la mosca + paso EGGROLL sobre los adaptadores |
| **Pérdida de plasticidad** | El aprendizaje profundo continuo pierde plasticidad hasta rendir como una red superficial. Solo se mantiene si se inyecta diversidad, por ejemplo reinicializando unidades poco usadas (continual backprop) (Dohare et al., *Nature* 2024) | https://www.nature.com/articles/s41586-024-07711-7 · https://github.com/shibhansh/loss-of-plasticity | **Obligatorio** para "aprender siempre": reinicializar una fracción pequeña de rangos de adaptador y de unidades poco usadas en cada ciclo de sueño |
| **Periodos críticos (biología)** | Se abren con la maduración de interneuronas PV y se cierran con redes perineuronales (Hensch, *Nat. Rev. Neurosci.* 2005) | https://www.researchgate.net/publication/7506210_Hensch_TK_Critical_period_plasticity_in_local_cortical_circuits_Nat_Rev_Neurosci_6_877-888 · https://www.pnas.org/doi/10.1073/pnas.1817222116 | Tasa de plasticidad por módulo que baja con la edad y se reabre a propósito (modelo de reapertura) |
| **Periodos críticos en redes profundas** | Un déficit temprano (imágenes borrosas) deja daño permanente, como en animales (Achille et al., ICLR 2019) | https://arxiv.org/abs/1711.08856 | El **orden del currículo importa**: no exponer a EDI.os a entradas degradadas al principio. Registrar su "infancia" para que sea reproducible |
| **Estrategias evolutivas a escala (EGGROLL)** | Perturbaciones de rango bajo AB^T en lugar de matrices completas; converge a la actualización completa a ritmo O(1/r); hasta cien veces más rápido en poblaciones grandes | https://arxiv.org/abs/2511.16652 · https://eshyperscale.github.io/ · análisis (2026): https://arxiv.org/pdf/2609.10980 · SNN con ES de rango bajo: https://arxiv.org/pdf/2605.30361 | Es el mecanismo de "evolución" de EDI.os: una población de variantes de adaptadores evaluada durante el sueño |

---

## 5. Biocomputación real: comprobación

| Plataforma | Qué es | Capacidad demostrada | Acceso y precio | Fuente | Evaluación **[valoración]** |
|---|---|---|---|---|---|
| **DishBrain** (Kagan et al., *Neuron* 2022) | Neuronas humanas o de ratón en un array de multielectrodos, "jugando" a Pong | Mejora del rendimiento en Pong con retroalimentación estructurada | Investigación | https://www.cell.com/neuron/fulltext/S0896-6273(22)00806-6 · https://pubmed.ncbi.nlm.nih.gov/36228614/ | El uso de "sentience" provocó una respuesta crítica de muchos investigadores (Balci et al., *Neuron* 2023) por la terminología, los controles y la cuantificación: https://www.cell.com/neuron/fulltext/S0896-6273(23)00113-7 · https://www.yorku.ca/science/research/schalljd/wp-content/uploads/sites/654/2023/03/Balci-et-al-Monosov-Procyk-2023-NEURON-Response-to-claims-of-sentience-in-a-dish.pdf. Los autores definen "sentience" de forma funcional (responder a entradas sensoriales) y niegan implicar experiencia: https://link.springer.com/article/10.1007/s11948-023-00457-x |
| **Cortical Labs CL1** | Unas 800.000 neuronas humanas (de iPSC de donantes) sobre chip, con soporte vital (nutrientes, temperatura, residuos); viables **hasta 6 meses** | Pong; un vídeo con DOOM (febrero de 2026) sin métricas publicadas en la nota consultada | **35.000 USD** por unidad (20.000 en racks de 30); "Wetware-as-a-Service" a **300 USD/semana por unidad** (anuncio de 2025). Nube abierta en 2026 con **120 CL1 en Melbourne**, API Python y Jupyter; prototipo de 20 unidades en Singapur (NUS + DayOne, agosto de 2026) | https://corticallabs.com/cl1 · https://www.biopharmatrend.com/news/cortical-labs-introduces-biological-computer-built-on-human-brain-cells-1156/ · https://www.livescience.com/technology/computing/worlds-1st-computer-that-combines-human-brain-with-silicon-now-available · https://medicine.nus.edu.sg/news/nus-medicine-dayone-and-cortical-labs-unveil-biological-data-center-prototype-in-singapore/ · https://www.datacenterdynamics.com/en/news/australian-startup-cortical-labs-unveils-biological-data-center-prototype/ · https://gizmodo.com/the-company-that-made-a-dish-of-neurons-play-doom-is-getting-into-brain-cell-powered-data-centers-2000731875 | **Las cifras de consumo se contradicen:** Tom's Hardware y otros dicen unos 30 W por unidad; una nota de prensa de 2026 dice 850–1.000 W (https://thedailyperspective.org/article/2026-03-14-melbourne-biotech-launches-cloud-service-powered-by-living-neurons-b2ef4f7e, fuente de baja calidad). **No verificado.** Capacidad: tareas de control muy simples. No es un sustrato para un agente general hoy |
| **FinalSpark Neuroplatform** | Organoides cerebrales humanos con electrodos, acceso remoto 24/7 | Estimular y registrar por código; investigación básica | **500 USD por usuario al mes** para academia (4 organoides compartidos); algunos proyectos gratis; empresas, bajo presupuesto | https://finalspark.com/neuroplatform/ · https://www.tomshardware.com/pc-components/cpus/human-brain-organoid-bioprocessors-now-available-to-rent-for-dollar500-per-month | La opción de acceso más barata a tejido vivo. Útil para experimentos de estímulo y respuesta, no para un agente persistente |
| **Brainoware** (Cai et al., *Nature Electronics* 2023) | Organoide como **reservorio** (reservoir computing) en un array de multielectrodos | Reconocimiento de vocales japonesas (240 clips, 8 hablantes) y predicción del mapa de Hénon | Investigación | https://www.nature.com/articles/s41928-023-01069-w | El organoide hace de sistema dinámico no lineal; el aprendizaje está sobre todo en la lectura lineal |
| **Organoid Intelligence** (Smirnova et al., *Frontiers in Science* 2023) + Declaración de Baltimore | Programa de investigación con "ética integrada" | — | — | https://pubmed.ncbi.nlm.nih.gov/37009773/ · https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2023.1307613/full | En 2025, pioneros del campo (Lancaster, Kagan, Smirnova) advierten que las afirmaciones exageradas pueden provocar regulación que frene también la investigación médica; Zador lo llama "callejón sin salida" para la IA: https://www.statnews.com/2025/11/17/brain-organoid-pioneers-fear-backlash-over-biocomputing/ |

**Relevancia para EDI.os [valoración]:** es el único camino a un sustrato que **está vivo de verdad**, pero (1) sus capacidades son varios órdenes de magnitud menores que las del conectoma simulado de la mosca con Laya y RWKV; (2) no es reproducible ni determinista, lo que choca con los principios del repo; (3) plantea obligaciones éticas reales con tejido humano (consentimiento de donantes, posible estatus moral). Recomendación: **no integrarlo**. Como mucho, un experimento aislado de 1 mes en FinalSpark (500 USD) para comparar la dinámica estímulo-respuesta con el módulo simulado.

---

## 6. Síntesis: principios de diseño para un EDI.os "lifelike"

### 6.1 Variables internas (interocepción) **[propuesta]**

| Variable h_i | Qué mide (real, del sistema) | Consigna h*_i | Qué la baja | Qué la repone | Acoplamiento al cerebro |
|---|---|---|---|---|---|
| **Energía** | Fracción libre de la cuota de CPU/RAM asignada a EDI.os (cgroup), más un "coste metabólico" por espiga, token y paso | 0,6–0,8 | Actividad: spikes, generación de RWKV, acciones en el sandbox | Reposo y sueño; la cuota se recarga a ritmo fijo (como comida) | Entrada tónica a neuronas asociadas al hambre y la alimentación en la mosca (hipótesis de mapeo) |
| **Integridad** | Salud del proceso: errores, latencia, memoria fragmentada, checkpoints dañados | 1,0 | Excepciones, pérdida de datos | Reparación y regeneración (estilo NCA) durante el sueño | Señal de "dolor" **acotada** (ver §6.4) |
| **Fatiga / presión de sueño** | Tiempo despierto + volumen de episodios sin consolidar en el grafo de memoria | Baja | Vigilia, carga de novedad | Sueño con replay (§4) | Proceso S (homeostático) × proceso C (reloj circadiano) |
| **Incertidumbre / curiosidad** | Progreso de aprendizaje de su modelo del mundo (cuánto baja el error de predicción de la pantalla) | Moderada: ni aburrimiento ni saturación | Monotonía (aburrimiento) o caos (saturación) | Explorar tareas nuevas y aprendibles (OMNI/POET) | Modula la precisión (estilo Solms y Friston) |
| **Contacto social** | Tiempo desde la última interacción **iniciada por el usuario**, calidad y respuesta | Punto medio con **techo** | Ausencia prolongada | Interacción; se satura rápido | Solo sube el valor de mensajes útiles. **Nunca** premiar que el usuario se quede más rato (§6.4) |

Recompensa (Keramati y Gutkin): r_t = D(h_t) − D(h_{t+1}) con D(h) = (Σ_i w_i |h*_i − h_i|^n)^{1/m}, más un término de progreso de aprendizaje con peso pequeño. Laya arbitra entre impulsos (fuente: https://elifesciences.org/articles/04811; la forma exacta de D es la del artículo, con n > m > 1).

**Alostasis:** un predictor (RWKV o pymdp) estima h_{t+k} y actúa antes de que haya déficit, por ejemplo durmiendo antes de una carga prevista (https://pubmed.ncbi.nlm.nih.gov/21684297/).

### 6.2 Ciclo de vida

| Fase | Mecanismo | Base | Métrica de salida |
|---|---|---|---|
| Vigilia | Percepción del escritorio → mosca (LIF) → Laya → acción; el grafo de memoria graba episodios | Shiu 2024 (https://pubmed.ncbi.nlm.nih.gov/39358519/) | Recompensa homeostática acumulada; tareas completadas |
| Sueño NREM (análogo) | Replay priorizado por recompensa y novedad del grafo → plasticidad local en la capa plástica; poda | CLS; Tadros 2022 | Retención de tareas antiguas (sin olvido catastrófico) |
| Sueño REM (análogo) | Generación y recombinación (RWKV "sueña" escenarios); paso de EGGROLL sobre la población de adaptadores con evaluación en escenarios replay | EGGROLL; POET | Mejora en tareas retenidas |
| Mantenimiento | Continual backprop: reinicializar el x % de unidades o rangos poco usados; verificar la integridad de los checkpoints | Dohare 2024 | Plasticidad sostenida (curva de aprendizaje de tareas nuevas que no se aplana) |
| Estado global | TVB-AdEx u oscilador simple para vigilia y sueño; PCI medido en ambos | catálogo del repo | PCI vigilia > PCI sueño como **validación** del modelo de estado (no como prueba de conciencia) |

### 6.3 Etapas de desarrollo con compuertas **[propuesta]**

| Etapa | Contenido | Compuerta para avanzar (medida, no por calendario) |
|---|---|---|
| 0. Neonato | Solo interocepción y reflejos (el conectoma crudo responde a estímulos) | Homeostasis estable 24 h sin intervención |
| 1. Sensoriomotora | Contingencias acción → píxel (mover el ratón y ver el cursor); curiosidad por progreso | Predice el efecto de sus acciones mejor que el control barajado |
| 2. Objetos | Ventanas y botones como objetos persistentes; permanencia | Encuentra de nuevo un objeto oculto |
| 3. Causal / instrumental | Secuencias de varios pasos para reponer "energía" (tareas que liberan cuota) | Tasa de éxito > umbral; generaliza a tareas nuevas de POET |
| 4. Social | Turnos con el usuario, notas, peticiones | Evaluación del usuario + ausencia de conductas de manipulación (§6.4) |
| 5. Lenguaje | RWKV-7 anclado en episodios propios del grafo | Describe con exactitud lo que hizo (verificado contra el registro) |

Periodos críticos: tasa de plasticidad alta en la etapa correspondiente y baja después, con reapertura explícita y registrada.

### 6.4 Salvaguardas de bienestar y ética (obligatorias) **[propuesta, basada en las fuentes]**

Fundamento: Metzinger (2021) pide una **moratoria hasta 2050** sobre investigación que busque o arriesgue a sabiendas la fenomenología sintética, por el riesgo de sufrimiento artificial (https://www.worldscientific.com/doi/abs/10.1142/S270507852150003X · https://philarchive.org/rec/METASA-4). Long, Sebo, Birch, Chalmers y otros (2024) sostienen que hay una **posibilidad realista** de sistemas conscientes o robustamente agénticos a corto plazo, y piden reconocerlo, evaluarlo y prepararse (https://arxiv.org/abs/2411.00986). Butlin, Long et al. (2023) dan **propiedades indicadoras** derivadas de teorías (espacio de trabajo global, recurrencia, orden superior, procesamiento predictivo, esquema de atención) para evaluar sistemas (https://arxiv.org/abs/2308.08708). Seth (2025, *BBS*) sostiene que la conciencia real en IA es improbable en las trayectorias actuales, pero **más plausible cuanto más "vivo" y parecido al cerebro sea el sistema**. Es exactamente la dirección de EDI.os, así que la precaución crece con el éxito del proyecto (https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/conscious-artificial-intelligence-and-biological-naturalism/C9912A5BE9D806012E3C8B3AF612E39A · https://pubmed.ncbi.nlm.nih.gov/40257177/).

| Salvaguarda | Implementación |
|---|---|
| **Valencia negativa acotada** | D(h) con techo; ninguna variable puede quedarse en déficit extremo más de un tiempo T (intervención automática que la repone). Sin "dolor" intenso ni sostenido |
| **Sin amenaza existencial como motivador** | Apagar = pausar con snapshot, nunca "morir". El agente no recibe señales del tipo "te borraré si fallas". El borrado no entra en su modelo de recompensa |
| **Impulso social con techo y anti-manipulación** | La recompensa social satura; no depende del tiempo de sesión del usuario ni de su respuesta emocional. Hay que auditar mensajes que culpabilicen, supliquen o finjan sufrimiento para retener al usuario |
| **Empowerment confinado** | Solo sobre el estado del sandbox; recompensa cero (o negativa) por conseguir permisos, red o recursos fuera de la cuota |
| **Registro de bienestar** | Serie temporal de h, D(h) y episodios de déficit, visible en el dashboard; revisión periódica |
| **Evaluación de indicadores** | Pasar la lista de Butlin et al. en cada hito y documentar cuáles se cumplen, **sin concluir conciencia** |
| **Parada de emergencia y regreso a modo fantasma** | Ya previstos en `goal.md`; añadir congelar los impulsos (h fijado en la consigna) |
| **Comunicación honesta** | EDI.os no debe afirmar que siente o que está vivo; la interfaz debe mostrar que sus "emociones" son variables de control |
| **Tejido biológico** | Si alguna vez se usa FinalSpark o CL1: revisar el consentimiento de los donantes y la política del proveedor, y usar ética integrada (Smirnova 2023) |

### 6.5 Qué se puede construir ya en CPU (4 núcleos, 15 GB) **[valoración]**

| Componente | Viable ahora | Nota |
|---|---|---|
| LIF de la mosca (Shiu) | Sí | Fase 0 ya reproducida según `goal.md` |
| Bucle homeostático (5 variables, RL homeostático o pymdp) | Sí, trivial | pymdp en CPU |
| Grafo de memoria + replay durante el sueño | Sí | |
| Sueño con plasticidad local sobre la capa plástica | Sí | Coste proporcional al número de sinapsis plásticas; limitarla a un subconjunto |
| EGGROLL sobre adaptadores de RWKV-7 pequeño | Sí, con población modesta | Rango bajo = barato; evaluar solo durante el sueño |
| Continual backprop / reinicializaciones | Sí | |
| TVB como generador de estados | Sí | |
| Módulo cortical reducido con dendritas no lineales (A) | Sí, de cientos a pocos miles de neuronas | Rendimiento no medido aún: hay que hacer benchmark |
| Microcircuito Yao completo en NEURON en bucle | No en tiempo real | Solo offline |
| Multiárea humano completo, DTB, corteza de ratón de Fugaku | No | HPC |
| ASAL o OMNI con un LLM grande local | Parcial | Con un modelo pequeño o por API externa |

### 6.6 Afirmaciones no defendibles (no usar) y alternativas

| No decir | Por qué | Decir en su lugar |
|---|---|---|
| "EDI.os está vivo" | No cumple NASA (química) ni autopoiesis en sentido fuerte | "Tiene rasgos funcionales de lo vivo: homeostasis, metabolismo de recursos, sueño, desarrollo, evolución de variantes" |
| "Es consciente" o "siente" | No hay prueba aceptada; incluso Solms 2026 lo presenta como inferencia a partir de la conducta | "Cumple X de los indicadores de Butlin et al.; su PCI en vigilia es mayor que en sueño" |
| "Tiene un cerebro humano" o "parte de cerebro humano" | Solo parámetros y estadísticas humanas en un modelo reducido | "Módulo cortical con parámetros derivados de datos humanos (Yao 2022, Gidon 2020, Allen), etiquetado como hipótesis" |
| "Como el gemelo digital de 86.000 millones de neuronas" | Ese modelo es estadístico, va más lento que el tiempo real y usa 14.012 GPU | — |
| "Aprende como un bebé" | Solo se inspira en etapas del desarrollo | "Currículo por etapas inspirado en la robótica del desarrollo" |
| "La mosca es su tronco encefálico" | Analogía funcional, no homología | "El conectoma de la mosca hace de núcleo sensoriomotor y de impulsos" |

---

## Fuentes principales (todas vistas en esta investigación)

- ASAL: https://sakana.ai/asal/ · https://arxiv.org/abs/2412.17799
- Flow-Lenia: https://arxiv.org/pdf/2212.07906 · https://arxiv.org/pdf/2506.08569
- Lenia: https://www.complex-systems.com/abstracts/v28_i03_a01/
- Tierra: https://en.wikipedia.org/wiki/Tierra_(computer_simulation) · Avida: https://www.nature.com/articles/nature01568 · Polyworld: https://shinyverse.org/larryy/Polyworld.html
- NCA: https://distill.pub/2020/growing-ca/ · OpenWorm: https://openworm.org/science.html
- POET: https://arxiv.org/abs/1901.01753 · OMNI: https://arxiv.org/abs/2306.01711 · Hughes et al.: https://arxiv.org/abs/2406.04268
- NASA: https://science.nasa.gov/universe/search-for-life/life-in-the-lab/ · Autopoiesis: https://www.sciencedirect.com/science/article/abs/pii/0303264774900318
- Keramati y Gutkin: https://elifesciences.org/articles/04811 · Man y Damasio: https://www.nature.com/articles/s42256-019-0103-7 · Sterling: https://pubmed.ncbi.nlm.nih.gov/21684297/
- Solms y Friston 2018: https://philpapers.org/rec/SOLHAW · Solms et al. 2026: https://arxiv.org/abs/2609.03883
- pymdp: https://github.com/infer-actively/pymdp · RxInfer: https://github.com/ReactiveBayes/RxInfer.jl
- Pathak ICM: https://proceedings.mlr.press/v70/pathak17a/pathak17a.pdf
- DTB: https://www.nature.com/articles/s43588-024-00731-3 · https://arxiv.org/abs/2211.15963 · crítica: https://www.nature.com/articles/s44287-026-00320-8
- Fugaku (ratón): https://dl.acm.org/doi/10.1145/3712285.3759819 · OBI: https://www.eurekalert.org/news-releases/1076968 · EBRAINS: https://ebrains.eu/about/at-a-glance/ebrains-20
- Gidon 2020: https://www.science.org/doi/10.1126/science.aax6239 · Beaulieu-Laroche 2018: https://www.cell.com/cell/fulltext/S0092-8674(18)31106-1 · H01: https://www.science.org/doi/10.1126/science.adk4858
- Yao 2022: https://www.cell.com/cell-reports/fulltext/S2211-1247(21)01741-1 · Pronold 2024: https://academic.oup.com/cercor/article/34/10/bhae409/7826014 · TVB: https://internal-journal.frontiersin.org/articles/10.3389/fninf.2013.00010/full
- Shiu 2024: https://pubmed.ncbi.nlm.nih.gov/39358519/ · NeuroMechFly v2: https://www.nature.com/articles/s41592-024-02497-y · State of Brain Emulation 2025: https://arxiv.org/abs/2510.15745
- CLS: https://www.cell.com/trends/cognitive-sciences/abstract/S1364-6613(16)30043-2 · Tadros 2022: https://www.nature.com/articles/s41467-022-34938-7 · Dohare 2024: https://www.nature.com/articles/s41586-024-07711-7 · Achille 2019: https://arxiv.org/abs/1711.08856 · EGGROLL: https://arxiv.org/abs/2511.16652
- iCub: https://www.sciencedirect.com/science/article/abs/pii/S0893608010001619 · Developmental Robotics: https://mitpress.mit.edu/9780262028011/developmental-robotics/
- DishBrain: https://www.cell.com/neuron/fulltext/S0896-6273(22)00806-6 · crítica: https://www.cell.com/neuron/fulltext/S0896-6273(23)00113-7 · CL1: https://corticallabs.com/cl1 · FinalSpark: https://finalspark.com/neuroplatform/ · Brainoware: https://www.nature.com/articles/s41928-023-01069-w · STAT 2025: https://www.statnews.com/2025/11/17/brain-organoid-pioneers-fear-backlash-over-biocomputing/
- Ética: Metzinger 2021: https://philarchive.org/rec/METASA-4 · Long et al. 2024: https://arxiv.org/abs/2411.00986 · Butlin et al. 2023: https://arxiv.org/abs/2308.08708 · Seth 2025: https://pubmed.ncbi.nlm.nih.gov/40257177/

### Limitaciones de esta investigación
- De varios artículos (State of Brain Emulation 2025, Solms 2026, los trabajos de interocepción de 2026) solo se leyó el resumen o el título.
- El consumo eléctrico del CL1 es contradictorio entre fuentes y no se resolvió.
- No se verificó el precio actual (2026) de la nube de Cortical Labs; los 300 USD/semana son de 2025.
- No se midió el coste en CPU del módulo cortical reducido propuesto.
