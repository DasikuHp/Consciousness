# Catálogo de recursos descargables — Consciousness

Fecha de verificación: 2026-09-27. Cada URL se abrió o apareció en resultados de búsqueda; cada repositorio se comprobó con `git ls-remote` o con un clon superficial (`--depth 1`) que se borró después. Las licencias se leyeron del archivo LICENSE o de los metadatos del paquete (PyPI, Zenodo, DANDI). **Tamaño "clon"** es lo que ocupa un `git clone --depth 1` (árbol de trabajo + `.git`), medido. Si algo no se pudo verificar, se dice.

Leyenda de prioridad: **P0** imprescindible · **P1** útil · **P2** referencia.
"¿Cabe en GitHub?": el código se instala como dependencia (pip/clon fuera del repo). En el repo solo van scripts de descarga y resultados pequeños. Límite de GitHub: 100 MB por archivo.

Restricción de cómputo: 4 CPU, 15 GB de RAM, sin GPU. **`df` marcaba 16 GB libres** en la fecha de verificación (el plan decía ~30 GB y el encargo ~25 GB).

---

## 0. Base de mosca (lo que el plan ya usa, con URLs concretas)

| Nombre | URL | Clon / descarga | Licencia | Tamaño | Uso en este proyecto | Prio | ¿GitHub? |
|---|---|---|---|---|---|---|---|
| FlyWire v783, conectividad (Zenodo) | https://zenodo.org/records/10676866 | descarga directa, **sin login** | CC-BY-4.0 | `proofread_connections_783.feather` 852 MB; `proofread_root_ids_783.npy` 1,1 MB; sinapsis individuales 9,5 GB (total 10,6 GB) | Descarga reproducible de la fase 0 sin cuenta de Codex. Incluye probabilidades de NT por arista | **P0** | No (script) |
| FlyWire annotations | https://github.com/flyconnectome/flywire_annotations | `git clone https://github.com/flyconnectome/flywire_annotations.git` | sin archivo LICENSE | clon 51 MB | Tipos celulares, `top_nt` por neurona, posiciones de soma para el dashboard 3D | **P0** | No (script) |
| FlyWire Codex | https://codex.flywire.ai/api/download | web (**requiere cuenta**) | — | "Connections (Unfiltered)" 277 MB | Alternativa a Zenodo. Muestra FAFB v783 (139.255 neuronas), BANC v888, MANC, MCNS v1.0 | P1 | No |
| MaleCNS v1.0 (archivos planos) | https://male-cns.janelia.org/download/ | `https://storage.googleapis.com/flyem-male-cns/v1.0/connectome-data/flat-connectome/connectome-weights-male-cns-v1.0-minconf-0.5.feather` (+ `body-annotations-…feather`, `body-neurotransmitters-male-cns-v1.0.feather`) | CC-BY 4.0 | pesos 1,05 GB (medido); NT por neurona 43 MB; anotaciones 13 MB; `syn-points` 12,7 GB y `syn-partners` 6,8 GB (**no caben juntos**) | Sustrato principal (fase 1): pesos agregados + NT | **P0** | No |
| BANC v888 | https://doi.org/10.7910/DVN/7WTH1N (Dataverse) · https://github.com/htem/BANC-project | Dataverse. **No clonar el repo entero: 2,8 GB** | CC BY 4.0 (datos y código) | metadatos 63 MB; resto en Dataverse/GCS | Segundo animal (fase 7). Incluye NT predichos y parquets de "influence" | P1 | No |
| Shiu et al. LIF | https://github.com/philshiu/Drosophila_brain_model | `git clone https://github.com/philshiu/Drosophila_brain_model.git` | MIT | clon 370 MB | Motor LIF de referencia (Brian2). Por defecto v630; trae `Connectivity_783.parquet` y `Completeness_783.csv` | **P0** | No (dependencia) |
| neuprint-python | https://github.com/connectome-neuprint/neuprint-python | `pip install neuprint-python` | BSD (HHMI) | clon 23 MB | Consultas a MaleCNS por neuPrint | P1 | Dependencia |
| CAVEclient | https://github.com/CAVEconnectome/CAVEclient | `pip install caveclient` | MIT | clon 9 MB | Acceso a FlyWire/BANC/MICrONS vía CAVE | P1 | Dependencia |
| navis | https://github.com/navis-org/navis | `pip install navis` | GPL-3.0 | clon 131 MB | Morfologías y esqueletos (SWC), visualización | P2 | Dependencia |

## 1. Humanos, a nivel de neurona o sinapsis

| Nombre | URL | Clon / descarga | Licencia | Tamaño | Uso en este proyecto | Prio | ¿GitHub? |
|---|---|---|---|---|---|---|---|
| H01 (Harvard/Google), ~1 mm³ de corteza temporal humana | https://h01-release.storage.googleapis.com/data.html | `gs://h01-release/data/20210601/proofread_104/skeletons/104_proofread_neurons_swc.zip` (también por HTTPS en storage.googleapis.com) | CC-BY 4.0 | SWC de 104 neuronas revisadas: **60 MB** (medido). Tabla de sinapsis C3 (Avro): **159 GB** (medido), **no cabe** | Morfologías humanas reales y estadísticas locales (E/I, tamaño de sinapsis) para comparar con la mosca. Solo por subconjuntos | P1 | No |
| Allen Cell Types Database (incluye humano) | https://celltypes.brain-map.org/ · SDK: https://github.com/AllenInstitute/AllenSDK | `pip install allensdk` | Allen Institute Software License (BSD-2 + cláusula); datos bajo los Terms of Use y la citation policy de Allen | clon del SDK 170 MB; datos por célula (pequeños) | Electrofisiología, morfología y modelos "Biophysical – all active" (NEURON) por célula. Da parámetros humanos medidos (`source: measured`) | P1 | No |
| Human interneuron patch-seq (DANDI 000636) | https://dandiarchive.org/dandiset/000636 · análisis: https://github.com/AllenInstitute/human_patchseq_gaba | `dandi download` de un subconjunto | CC-BY-4.0 (DANDI); código con licencia Allen | **24,5 GB, 706 NWB** (medido vía API), **no cabe entero**; repo 64 MB | Ancla electrofisiológica humana por tipo transcriptómico (Lamp5/Pvalb/Sst/Vip) | P2 | No |
| Microcircuito humano L2/3 (Yao et al. 2022) | https://modeldb.science/267595 · https://github.com/KantYao/Human-L2-3-Cortical-Microcircuit | `git clone https://github.com/ModelDBRepository/267595.git` | **sin archivo LICENSE** | clon 7 MB | 1.000 neuronas humanas detalladas (Pyr/SST/PV/VIP) en NEURON+LFPy, con bandera `MDD`. Da un contraste humano de persistencia y respuesta a estímulo en CPU | P1 | Dependencia (sin licencia: no vendorizar) |
| Human multi-area model (Pronold et al. 2024) | https://github.com/INM-6/human-multi-area-model | `git clone https://github.com/INM-6/human-multi-area-model.git` | GPL-3.0 | clon 34 MB | Modelo spiking de 34 áreas humanas (3,47 M neuronas, 42,8 mil M sinapsis, NEST). A escala completa es de HPC. Sirve de referencia de conectividad por capa | P2 | Dependencia |
| The Virtual Brain (TVB) | https://github.com/the-virtual-brain/tvb-root | `pip install tvb-library` (2.10.0) | GPL-3.0 | clon 461 MB | Cerebro humano a macroescala (masas neuronales) + conectoma estructural. Barato en CPU | P1 | Dependencia |
| tvb-data (conectomas de demostración) | https://zenodo.org/records/10128131 | `tvb_data.zip` | GPL (el repo GitHub `tvb-data` está **archivado**; usar Zenodo) | 337 MB | Conectoma humano por defecto (`connectivity_76`), macaco y ratón | P1 | No |
| **TVB-AdEx + PCI** (Goldman et al. 2023, Front. Comput. Neurosci.) | https://gitlab.ebrains.eu/kancourt/tvb-adex-showcase3-git | `git clone https://gitlab.ebrains.eu/kancourt/tvb-adex-showcase3-git.git` | **sin archivo LICENSE** | clon 20 MB | Único código verificado que calcula PCI en un cerebro humano simulado con estados tipo vigilia y sueño. Trae `pci_v2.py` y `Lempel_Ziv.py`. Es el comparador directo del marcador PCI | **P0** (referencia metodológica) | Dependencia (sin licencia: no copiar código) |
| Human Connectome Project | https://www.humanconnectome.org/study/hcp-young-adult/data-use-terms · https://github.com/datalad-datasets/human-connectome-project-openaccess | registro en ConnectomeDB + aceptar los Open Access Data Use Terms | términos propios HCP (no CC) | TB en crudo | Solo macroconectividad (dMRI). Con tvb-data basta para este proyecto | P2 | No |
| BigBrain | https://bigbrain-ftp.loris.ca/ · https://bigbrainproject.org/ | FTP (volúmenes de 100–400 µm en MINC/NIfTI) | según la política de citación del sitio | 20 µm completo: "demasiado grande para transferencias" | Citoarquitectura; sin sinapsis | P2 | No |
| NeuroMorpho.org | https://neuromorpho.org/ | API / SWC | según el dataset | variable | Morfologías humanas adicionales | P2 | No |

## 2. Otros conectomas completos o casi completos

| Nombre | URL | Clon / descarga | Licencia | Tamaño | Uso en este proyecto | Prio | ¿GitHub? |
|---|---|---|---|---|---|---|---|
| **C. elegans Connectome Toolbox (`cect`)** — Cook 2019 (ambos sexos), Witvliet 2021 (8 individuos, L1→adulto), White 1986… | https://openworm.org/ConnectomeToolbox/ · https://github.com/openworm/ConnectomeToolbox | `pip install cect` (0.3.4) | MIT (código) | clon 238 MB; pip pequeño | Conectomas de ~300 neuronas con sinapsis químicas y eléctricas: todo el pipeline (LIF, PCI, PyPhi en subredes) corre en segundos. Control entre especies y entre individuos | **P0** | Dependencia |
| c302 (OpenWorm) | https://github.com/openworm/c302 | `git clone https://github.com/openworm/c302.git` | MIT | clon 144 MB | Genera modelos NeuroML del gusano a varios niveles de detalle | P1 | Dependencia |
| BAAIWorm | https://github.com/Jessie940611/BAAIWorm | `git clone https://github.com/Jessie940611/BAAIWorm.git` | Apache-2.0 | clon 609 MB | Gusano con cerebro multicompartimental + cuerpo + entorno (Nat. Comput. Sci. 2024) | P2 | No |
| **Larva de Drosophila (Winding et al. 2023, Science)** | https://github.com/brain-networks/larval-drosophila-connectome | `git clone https://github.com/brain-networks/larval-drosophila-connectome.git` | **sin archivo LICENSE** (espejo de los datos suplementarios) | clon 3 MB | 3.016 neuronas y ~548k sinapsis: cerebro de insecto completo y barato para depurar y para el control "otra etapa vital" | **P0** | Pequeño; mejor script |
| MICrONS (ratón, corteza visual, mm³) | https://www.microns-explorer.org/cortical-mm3 · https://tutorial.microns-explorer.org/ | CAVEclient, datastack `minnie65_public` (hay que aceptar los ToS de CAVE) | citation policy + ToS de MICrONS | 523 M sinapsis (no descargar entero); subconjuntos de neuronas revisadas | Conectividad de mamífero con tipos celulares y actividad funcional del mismo tejido (Nature 2025) | P1 | No |
| Allen biorealistic V1 (sucesor de Billeh 2020) | https://github.com/AllenInstitute/biorealistic-v1-model · https://portal.brain-map.org/explore/models/mv1-all-layers | `git clone https://github.com/AllenInstitute/biorealistic-v1-model.git` | BSD-3-Clause | clon 8 MB; redes precompiladas en Dropbox (tamaño no verificado) | V1 de ratón con GLIF (BMTK/NEST) y datos de MICrONS | P1 | Dependencia |
| BBP "Model of Rodent Neocortical Micro- and Mesocircuitry" (Isbister et al.) | https://zenodo.org/records/11113043 | `O1_data.xz` | **CC BY-NC 4.0** | **51,6 GB, no cabe** | Circuito de 211.712 neuronas de corteza **de rata** (SONATA/Neurodamus). Solo referencia | P2 | No |
| BBP Canonical Electrical Neuron Models | https://zenodo.org/records/15006076 | `emodels.zip` | CC BY 4.0 | 10,8 MB | 16 e-modelos NEURON (corteza somatosensorial + tálamo, roedor) | P2 | Sí (pequeño) |
| Neurodamus | https://github.com/openbraininstitute/neurodamus | clon | Apache-2.0 | clon 36 MB | Simulador de SONATA de BBP. **`BlueBrain/neurodamus` está archivado; usar el fork de OBI** | P2 | Dependencia |
| Pez cebra larval (Petkova et al., bioRxiv 2025; Fish-X, bioRxiv 2025) | https://www.biorxiv.org/content/10.1101/2025.06.10.658982v1 · https://www.biorxiv.org/content/10.1101/2025.06.12.659365v2 | **no se encontró enlace de descarga verificable** ("plataforma de acceso abierto" sin URL en el texto consultado) | — | Petkova: >40.000 neuronas y 30 M sinapsis | Vertebrado cerebro completo. Seguir hasta que publique tablas | P2 | — |
| Platynereis (larva de 3 días, cuerpo entero; eLife 2025) | https://jekelylab.github.io/Platynereis_connectome/ · CATMAID: https://catmaid.jekelylab.ex.ac.uk · código: https://github.com/JekelyLab/Platynereis_3D_connectome_2024 | API de CATMAID (natverse) | GPL-3.0 (código) | **clon 4,2 GB: no clonar**; usar la API | Conectoma de animal entero, con circuitos peptidérgicos | P2 | No |
| Ciona intestinalis larva (Ryan et al. 2016, eLife) | https://elifesciences.org/articles/16962 | tabla suplementaria (Figure 1—figure supplement 2) | CC-BY (eLife) | pequeño | 177 neuronas del SNC, 6.618 sinapsis y 1.206 uniones gap. Cordado mínimo. (Existe `CionaBrain/CionaBrain` en GitHub, creado 2026-09-11, no oficial: ignorar) | P2 | Sí |
| BANC (publicado en Nature, 2026-06) | ver §0 | — | — | — | DOI 10.1038/s41586-026-10735-w | — | — |

## 3. Herramientas de conciencia y complejidad con código

| Nombre | URL | Clon / instalación | Licencia | Tamaño | Uso en este proyecto | Prio | ¿GitHub? |
|---|---|---|---|---|---|---|---|
| **PCIst** (Comolatti et al. 2019) | https://github.com/renzocom/PCIst | `git clone https://github.com/renzocom/PCIst.git` (**no está en PyPI**) | **GPL-3.0** | clon 1 MB | PCI por transiciones de estado sobre respuestas evocadas (tasas por región) | **P0** | Dependencia (GPL: no copiar a un repo MIT/Apache) |
| **pypci** (PCI-LZ, Casali 2013) | https://github.com/noreun/pypci | `git clone https://github.com/noreun/pypci.git` | **sin licencia** (solo lectura y uso local) | clon 3 MB | PCI clásico con Lempel-Ziv binarizado; segundo estimador para contrastar con PCIst | **P0** | No redistribuir |
| **AntroPy** (`lziv_complexity`, entropías) | https://github.com/raphaelvallat/antropy | `pip install antropy` (0.2.2) | BSD-3 | clon 3 MB | LZ normalizado, entropía de permutación y espectral. Implementación permisiva para escribir PCI-LZ propio | **P0** | Dependencia |
| **PyPhi** (IIT 3.0; IIT 4.0 en rama) | https://github.com/wmayner/pyphi | `pip install pyphi` → **1.2.0 = IIT 3.0**; IIT 4.0: `git clone -b feature/iit-4.0 https://github.com/wmayner/pyphi.git` | **GPL-3.0** | clon 187 MB | Φ exacto en microcircuitos de ~≤10 nodos extraídos del conectoma | **P0** | Dependencia |
| phyid (Integrated Information Decomposition, ΦID) | https://github.com/Imperial-MIND-lab/integrated-info-decomp | `git clone https://github.com/Imperial-MIND-lab/integrated-info-decomp.git` (no está en PyPI) | BSD-3 | clon 1 MB | Sinergia/redundancia entre regiones (enfoque de "synergistic workspace"). Escala mejor que PyPhi | P1 | Dependencia |
| PhiToolbox (Φ\*, Φ_G, búsqueda de MIP y complejos) | https://github.com/oizumi-lab/PhiToolbox | clon | **sin licencia**; MATLAB | clon 219 MB | Aproximaciones gaussianas de Φ; requiere MATLAB | P2 | No |
| JIDT (entropía de transferencia, información mutua) | https://github.com/jlizier/jidt | clon / jar | GPL-3.0 | clon 65 MB | Flujo de información dirigido entre módulos y Laya ↔ cerebro | P1 | Dependencia |
| Elephant (análisis de spikes: ensambles CAD/SPADE/ASSET, sincronía) | https://github.com/NeuralEnsemble/elephant | `pip install elephant` | BSD-3 | clon 8 MB | Detectar ensambles/atractores en los raster | P1 | Dependencia |
| Bazhenov: sleep-stage-transition (Krishnan et al. 2016, eLife) | https://github.com/bazhlab-ucsd/sleep-stage-transition | clon (C++) | **sin licencia** | clon 1 MB | Tálamo-corteza con ACh/histamina/GABA: transiciones vigilia-sueño por neuromodulación | P1 | No copiar |
| Bazhenov: whole-brain thalamocortical | https://github.com/bazhlab-ucsd/whole-brain-public | clon (C++) | **sin licencia** | clon 1 MB | Modelo tálamo-cortical a gran escala | P2 | No copiar |
| Versión NEURON/Python del modelo de sueño (Fink et al. 2024) | https://modeldb.science/2016601 | ModelDB | ver ModelDB | — | Mismo modelo en NEURON | P2 | — |
| Modelos de anestesia en masa media (Nat. Comput. Sci. 2025, s43588-025-00796-8) | https://www.nature.com/articles/s43588-025-00796-8 | **disponibilidad de código no verificada** (redirección a login) | — | — | Anestésicos GABA_A/NMDA → ondas lentas; perturbación con PCI | P2 | — |
| Global Neuronal Workspace (Dehaene–Changeux) spiking | — | **no se encontró código público** en ModelDB ni en GitHub | — | — | Hay que implementar la "ignición" como métrica propia | — | — |

## 4. Simuladores y marcos baratos

| Nombre | URL | Instalación | Licencia | Clon | Uso en este proyecto | Prio | ¿GitHub? |
|---|---|---|---|---|---|---|---|
| **Brian2** | https://github.com/brian-team/brian2 | `pip install brian2` (2.10.1) | CeCILL-2.1 | 9 MB | Motor de Shiu; generación de C++ en CPU | **P0** | Dependencia |
| **pycma** (CMA-ES) | https://github.com/CMA-ES/pycma | `pip install cma` (4.5.0) | BSD-3 | 9 MB | Evolución del "genoma" (10²–10³ parámetros) | **P0** | Dependencia |
| NEST | https://github.com/nest/nest-simulator | `pip install nest-simulator` (3.10.0 en PyPI) | GPL-2.0 | 93 MB | LIF multihilo en CPU; necesario para el modelo multiárea humano y Allen V1 | P1 | Dependencia |
| BrainPy | https://github.com/brainpy/BrainPy | `pip install brainpy` (2.8.2) | GPL-3.0 | 63 MB | Dinámica cerebral en JAX; alternativa rápida en CPU al LIF disperso | P1 | Dependencia |
| jaxley | https://github.com/jaxleyverse/jaxley | `pip install jaxley` (0.14.0) | Apache-2.0 | 9 MB | Biofísica diferenciable en CPU; ajustar parámetros por gradiente en subcircuitos | P1 | Dependencia |
| NEURON | https://github.com/neuronsimulator/nrn | `pip install neuron` (9.0.2) | BSD-3 | 61 MB | Necesario para Yao L2/3, Allen all-active y e-modelos BBP | P1 | Dependencia |
| libNeuroML / pyNeuroML | https://github.com/NeuralEnsemble/libNeuroML · https://github.com/NeuroML/pyNeuroML | `pip install libNeuroML pyNeuroML` | BSD-2 / LGPL-3.0 | 18 / 64 MB | Formato de intercambio (c302, Open Source Brain) | P1 | Dependencia |
| e2l-cgp-snn (Jordan et al., reglas de plasticidad evolucionadas con CGP) | https://github.com/Happy-Algorithms-League/e2l-cgp-snn | clon | GPL-3.0 | 75 MB | Precedente directo de "regla de plasticidad evolucionada" en redes spiking | P1 | Dependencia |
| fSBI (Confavreux et al. NeurIPS 2023, metaaprendizaje de reglas de plasticidad con SBI) | https://github.com/VogelsLab/fSBI | clon | **sin licencia** | 258 MB | Alternativa a ES para inferir familias de reglas | P2 | No copiar |
| Nengo / NengoSPA | https://github.com/nengo/nengo · https://github.com/nengo/nengo-spa | `pip install nengo nengo-spa` (4.1.0) | **GPL-2.0** (ya no es la licencia no comercial antigua) | 4 / 2 MB | Arquitecturas cognitivas (SPA); útil para prototipar un "workspace" | P2 | Dependencia |
| Arbor | https://github.com/arbor-sim/arbor | `pip install arbor` (0.12.2) | BSD-3 | 65 MB | Multicompartimental de alto rendimiento (aparcado) | P2 | Dependencia |
| GeNN | https://github.com/genn-team/genn | desde fuente (`pygenn` no está en PyPI) | LGPL-2.1 | 9 MB | Orientado a GPU (CUDA/HIP); poco valor sin GPU | P2 | — |
| Norse | https://github.com/norse/norse | `pip install norse` (1.1.0) | LGPL-3.0 | 5 MB | SNN en PyTorch (deep learning) | P2 | — |
| snnTorch | https://github.com/jeshraghian/snntorch | `pip install snntorch` (1.0.0) | MIT | 86 MB | SNN en PyTorch | P2 | — |
| SpikingJelly | https://github.com/fangwei123456/spikingjelly | `pip install spikingjelly` | licencia OpenI 启智 1.0 (no estándar: revisar) | 102 MB | SNN en PyTorch | P2 | — |
| BindsNET | https://github.com/BindsNET/bindsnet | pip | **AGPL-3.0** | 58 MB | SNN con STDP en PyTorch | P2 | — |
| BMTK (Allen) | https://github.com/AllenInstitute/bmtk | `pip install bmtk` (1.2.0) | BSD (Allen) | 413 MB | Construir y simular redes SONATA (Allen V1) | P2 | — |

## 5. Neurotransmisores, neuropéptidos y receptores (mosca)

| Nombre | URL | Descarga | Licencia | Tamaño | Uso en este proyecto | Prio | ¿GitHub? |
|---|---|---|---|---|---|---|---|
| NT por neurona MaleCNS | `…/body-neurotransmitters-male-cns-v1.0.feather` (ver §0) | GCS | CC-BY 4.0 | 43 MB | Signo sináptico con probabilidad (sustituye a "GABA/Glu = −1") | **P0** | No |
| NT en FlyWire (Eckstein et al. 2024, Cell) | https://zenodo.org/records/10676866 (probabilidades por arista) · https://github.com/flyconnectome/flywire_annotations (`top_nt`) | ver §0 | CC-BY-4.0 | ver §0 | Igual, para FAFB v783 | **P0** | No |
| Ground truth de neurotransmisores | https://github.com/flyconnectome/drosophila_neurotransmitters | `git clone https://github.com/flyconnectome/drosophila_neurotransmitters.git` | CC BY 4.0 | 114 MB | Tipos con NT confirmado experimentalmente: marca `measured` frente a `predicted` | P1 | Pequeño |
| synister (clasificador de NT) | https://github.com/funkelab/synister | clon | sin archivo LICENSE | 66 MB | Solo si hay que re-predecir NT | P2 | — |
| NT de BANC (synister_banc) | https://doi.org/10.7910/DVN/7WTH1N | Dataverse | CC BY 4.0 | — | Signo en BANC | P1 | No |
| Fly Cell Atlas (snRNA-seq) | https://flycellatlas.org/ · ArrayExpress E-MTAB-10519 | h5ad / loom | licencia no indicada en la página consultada | — | Expresión de receptores (DopR, GABA-B, receptores de neuropéptidos…) por tipo celular, para marcar receptores como `hypothesis` o `measured` | P1 | No |
| Conectoma neurosecretor de FlyWire (80 NSC por neuropéptido) | https://elifesciences.org/reviewed-preprints/102684 · https://pmc.ncbi.nlm.nih.gov/articles/PMC11384003/ | suplementos | CC-BY (eLife) | pequeño | Identidad peptidérgica de las neurosecretoras (canal hormonal) | P2 | Sí |
| Chemoconnectome (Deng et al. 2019, Neuron) | https://bdsc.indiana.edu/stocks/misc/chemcct.html | líneas de moscas, **no es una base de datos descargable** | — | — | Referencia de los 193 genes de transmisión química | P2 | — |
| **Laguna** | — | — | — | — | **No se encontró una tabla descargable de "conectoma de receptores" o neuropéptidos para todo el cerebro de la mosca.** Los NT disponibles son rápidos (ACh/GABA/Glu/DA/5-HT/OA…) y predichos | — | — |

---

## 6. Qué corrige este catálogo respecto al plan (`goal.md`)

1. **Licencias GPL en herramientas "núcleo".** PCIst (GPL-3.0), PyPhi (GPL-3.0), TVB (GPL-3.0), NEST (GPL-2.0), navis (GPL-3.0) y BrainPy (GPL-3.0). Si el repo va a ser MIT/Apache, se usan como dependencias instaladas y **no se copia su código**. PCIst no está en PyPI, así que hay que clonarlo fuera del repo o reescribir el algoritmo desde el paper. pypci, TVB-AdEx, Yao L2/3, Bazhenov, PhiToolbox y fSBI **no tienen licencia**: solo se pueden leer y usar en local.
2. **PyPhi desde pip es IIT 3.0** (1.2.0). IIT 4.0 solo está en la rama `feature/iit-4.0`.
3. **Descarga de FlyWire:** Codex exige cuenta. Zenodo 10676866 (CC-BY, sin login) permite una descarga reproducible sin credenciales. El archivo de sinapsis individuales (9,5 GB) no hace falta en la fase 0.
4. **Disco:** el plan dice ~30 GB, pero `df` marcaba 16 GB libres. Los `syn-points` (12,7 GB) y `syn-partners` (6,8 GB) de MaleCNS no caben juntos. Usar `connectome-weights` (1,05 GB). **No clonar** `htem/BANC-project` entero (2,8 GB) ni `JekelyLab/Platynereis_3D_connectome_2024` (4,2 GB).
5. **Cifras de BANC:** el README dice ~188.000 neuronas y 199 M sinapsis predichas (coincide con el plan), pero Codex lista "BANC v888 (CNS)" con 158.262 neuronas y phys.org dice ~160.000. Hay que documentar qué subconjunto se usa. BANC ya está **publicado** (Nature, 2026-06-14, DOI 10.1038/s41586-026-10735-w): conviene citarlo en vez del blog.
6. **"Base de neurotransmisores" era vaga.** Las fuentes concretas están en §5. No existe una base de receptores o neuropéptidos a nivel de conectoma: la sección de riesgos sobre el signo sin receptor sigue vigente.
7. **Repos movidos o archivados:** `the-virtual-brain/tvb-data` está archivado (usar Zenodo 10128131) y `BlueBrain/neurodamus` también (usar `openbraininstitute/neurodamus`).
8. **GNW:** no hay implementación spiking pública verificada del modelo Dehaene–Changeux. Si se mide "ignición/broadcast", la métrica es propia y se marca `hypothesis`.
9. **Nengo** ahora es GPL-2.0 (ya no tiene la licencia no comercial antigua). **NEST** se instala con `pip install nest-simulator`, así que retomarlo cuesta menos de lo que suponía el plan.
10. Correcto en el plan y confirmado: FAFB v783 con 139.255 neuronas; MaleCNS con 166.700 neuronas, CC-BY y paper en Cell (2026-09-03); Shiu por defecto en v630 con archivos para v783 (MIT); `eonsystemspbc/fly-brain` en GPL-2.0; `Kisame76/drosophila-brain-mlx` en MIT con control barajado. **No verificado aquí:** el número de tipos de MaleCNS (11.710), la F1 ≈ 0,83 de BANC y los datos de Laya y RWKV.

---

## Fuentes

- FlyWire Zenodo v783: https://zenodo.org/records/10676866 · Codex: https://codex.flywire.ai/api/download · anotaciones: https://github.com/flyconnectome/flywire_annotations
- MaleCNS: https://male-cns.janelia.org/download/ · https://www.janelia.org/project-team/flyem/male-cns-connectome
- BANC: https://github.com/htem/BANC-project · https://doi.org/10.7910/DVN/7WTH1N · https://phys.org/news/2026-06-publish-connectome-fruit-fly-brain.html
- Shiu: https://github.com/philshiu/Drosophila_brain_model
- H01: https://h01-release.storage.googleapis.com/data.html · https://h01-release.storage.googleapis.com/landing.html · https://research.google/blog/ten-years-of-neuroscience-at-google-yields-maps-of-human-brain/
- Allen Cell Types: https://celltypes.brain-map.org/ · https://allensdk.readthedocs.io/en/latest/biophysical_models.html · https://alleninstitute.org/legal/terms-of-use
- DANDI 000636: https://dandiarchive.org/dandiset/000636 · https://github.com/AllenInstitute/human_patchseq_gaba
- Yao 2022: https://modeldb.science/267595 · https://github.com/KantYao/Human-L2-3-Cortical-Microcircuit
- Human multi-area model: https://github.com/INM-6/human-multi-area-model · https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11491286/
- TVB: https://github.com/the-virtual-brain/tvb-root · https://zenodo.org/records/10128131 · https://github.com/the-virtual-brain/tvb-data
- TVB-AdEx/PCI: https://www.frontiersin.org/journals/computational-neuroscience/articles/10.3389/fncom.2022.1058957/full · https://gitlab.ebrains.eu/kancourt/tvb-adex-showcase3-git
- HCP: https://www.humanconnectome.org/study/hcp-young-adult/data-use-terms · BigBrain: https://bigbrain-ftp.loris.ca/bigbrain-ftp/FAQ.html · NeuroMorpho: https://neuromorpho.org/
- C. elegans: https://openworm.org/ConnectomeToolbox/ · https://github.com/openworm/c302 · https://github.com/Jessie940611/BAAIWorm · https://www.nature.com/articles/s43588-024-00738-w
- Larva de mosca: https://github.com/brain-networks/larval-drosophila-connectome · https://www.science.org/doi/10.1126/science.add9330
- MICrONS: https://www.microns-explorer.org/cortical-mm3 · https://tutorial.microns-explorer.org/quickstart_notebooks/04-cave-query-synapses.html
- Allen V1: https://github.com/AllenInstitute/biorealistic-v1-model · https://portal.brain-map.org/explore/models/mv1-all-layers
- BBP: https://zenodo.org/records/11113043 · https://zenodo.org/records/15006076 · https://github.com/openbraininstitute/neurodamus · https://www.openbraininstitute.org/about
- Pez cebra: https://www.biorxiv.org/content/10.1101/2025.06.10.658982v1 · https://www.biorxiv.org/content/10.1101/2025.06.12.659365v2.full
- Platynereis: https://jekelylab.github.io/Platynereis_connectome/ · https://github.com/JekelyLab/Platynereis_3D_connectome_2024 · Ciona: https://elifesciences.org/articles/16962
- PCIst: https://github.com/renzocom/PCIst · pypci: https://github.com/noreun/pypci · AntroPy: https://raphaelvallat.com/antropy/ · PyPhi: https://github.com/wmayner/pyphi (rama `feature/iit-4.0`) · phyid: https://github.com/Imperial-MIND-lab/integrated-info-decomp · PhiToolbox: https://github.com/oizumi-lab/PhiToolbox · JIDT: https://github.com/jlizier/jidt · Elephant: https://github.com/NeuralEnsemble/elephant
- Bazhenov: https://github.com/bazhlab-ucsd/sleep-stage-transition · https://github.com/bazhlab-ucsd/whole-brain-public · https://modeldb.science/2016601 · https://elifesciences.org/articles/18607
- Anestesia: https://www.nature.com/articles/s43588-025-00796-8 · GNW (sin código): https://pmc.ncbi.nlm.nih.gov/articles/PMC8770991/
- Simuladores: https://github.com/brian-team/brian2 · https://github.com/nest/nest-simulator · https://github.com/genn-team/genn · https://github.com/norse/norse · https://github.com/jeshraghian/snntorch · https://github.com/BindsNET/bindsnet · https://github.com/nengo/nengo · https://github.com/nengo/nengo-spa · https://github.com/NeuralEnsemble/libNeuroML · https://github.com/NeuroML/pyNeuroML · https://github.com/arbor-sim/arbor · https://github.com/neuronsimulator/nrn · https://github.com/jaxleyverse/jaxley · https://github.com/brainpy/BrainPy · https://github.com/fangwei123456/spikingjelly · https://github.com/AllenInstitute/bmtk · https://github.com/CMA-ES/pycma
- Plasticidad: https://github.com/Happy-Algorithms-League/e2l-cgp-snn · https://github.com/VogelsLab/fSBI
- NT/neuropéptidos: https://www.cell.com/cell/fulltext/S0092-8674(24)00307-6 · https://github.com/flyconnectome/drosophila_neurotransmitters · https://github.com/funkelab/synister · https://flycellatlas.org/ · https://elifesciences.org/reviewed-preprints/102684 · https://bdsc.indiana.edu/stocks/misc/chemcct.html
