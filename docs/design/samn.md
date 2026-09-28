# SAMN — Semantic Associative Memory Network

> Memoria asociativa semántico-episódica basada en grafos dinámicos y activación propagada. Teoría original del autor del proyecto; este documento es la **primera formalización** para EDI.os (v0.1, 2026-09-28).
> Se apoya en piezas existentes y verificadas: activación propagada con decaimiento e inhibición lateral (SYNAPSE, ACL 2026), grafo hebbiano con consolidación episódica→semántica (HeLa-Mem, ACL 2026), base-level learning (ACT-R), repetición IER/SER y sistemas de aprendizaje complementarios (hipocampo/neocórtex). **Lo nuevo** es cómo se acoplan: la memoria está unida al cerebro de mosca (cuerpo fungiforme), fundamenta el lenguaje y dirige la evolución.

---

## 1. Papel en EDI

```
   cerebro-mosca (= RWKV sobre conectoma)            Laya = hipocampo + prefrontal
          │   estado neuronal                          │  ¿codificar? ¿consolidar?
          ▼                                            ▼  ¿recuperar? ¿actuar?
   ┌──────────────────────────── SAMN ─────────────────────────────┐
   │  EPISÓDICA  →  SEMÁNTICA  →  PROCEDURAL   (+ LÉXICO, + YO)     │
   │  grafo dinámico · pesos hebbianos · activación propagada        │
   └───────┬──────────────────────┬──────────────────────┬──────────┘
           │ top-down al cuerpo   │ significado de       │ máscara y rango
           │ fungiforme (MB)      │ las palabras         │ para EGGROLL
           ▼                      ▼                      ▼
     percepción/acción       habla (RWKV)            aprendizaje
```

La SAMN es el **hipocampo y neocórtex asociativo** de EDI. En la mosca, el centro de memoria asociativa es el **cuerpo fungiforme** (células de Kenyon → MBON, con dopamina como señal de valor). La SAMN se conecta ahí: recibe el patrón de las KC y devuelve activación top-down a las MBON y DAN. Es una extensión, no un sustituto.

## 2. Nodos

| Tipo | Qué representa | Ejemplo |
|---|---|---|
| `Episode` | Un tramo de vida con inicio y fin | "El usuario abrió el editor y me habló" |
| `Event` | Suceso dentro de un episodio | `CLICK(botón=Guardar)`, `PUERTO_ABIERTO(443)` |
| `Percept` | Prototipo perceptivo (patrón neuronal recurrente) | patrón visual "ventana", patrón auditivo "voz del usuario" |
| `Concept` | Abstracción consolidada de muchos episodios | "cuando el usuario dice 'guarda', luego aparece un diálogo" |
| `Skill` | Secuencia de acciones reutilizable | "abrir menú → Guardar → Enter" |
| `Word` | Unidad léxica anclada | "guardar" ↔ `Concept(guardar)` |
| `Entity` | Algo persistente del mundo | el usuario, una app, un archivo, un proceso |
| `Self` | Modelo de sí mismo (un único nodo con subnodos) | "tengo sueño", "mi atención está en X" |

**Estado de cada nodo:**
- `a` — activación actual en [0,1];
- `B` — fuerza de base (ACT-R): `B = ln Σ_k (t − t_k)^(−d)`, donde `t_k` son los accesos pasados;
- `v` — valencia en [−1,1], ligada al afecto homeostático;
- `c` — confianza;
- `src` — `measured | inferred | dreamed`, clave para el monitor de realidad HOT-2;
- `emb` — vector del estado neuronal en el que se formó (huella del cerebro).

## 3. Aristas

Tipos: `temporal`, `causal`, `contains`, `similar`, `contrast`, `supports`, `predicts`, `grounds` (palabra↔concepto), `involves` (entidad), `about_self`.

Cada arista tiene peso `w`, tipo, última coactivación y evidencia (número de episodios que la sostienen).

**Plasticidad:**
- **Hebbiana con olvido:** `Δw_ij = η·a_i·a_j − λ·w_ij`.
- **Causal (tipo STDP):** si `i` precede a `j` dentro de una ventana τ, entonces `Δw_ij^causal = η_c·e^(−Δt/τ)`. Si el orden se invierte, la arista se debilita.
- **Modulación por valor:** `η` se escala con la señal dopaminérgica del cerebro (DAN del MB) y con la sorpresa, así que se aprende más de lo sorprendente y lo valioso.
- **Reconsolidación:** al recuperar un recuerdo, sus aristas quedan lábiles durante una ventana y pueden reescribirse con la nueva experiencia.

## 4. Recuperación: activación propagada

1. **Semillas:** el estado actual (patrón de KC, contenido del workspace, palabras oídas o leídas, eventos del sistema) activa los nodos más similares por `emb`.
2. **Propagación** durante T pasos:
   `a_j(t+1) = σ( (1−δ)·a_j(t) + Σ_i a_i(t)·w_ij·g(tipo_ij) + β·B_j − θ )`
3. **Inhibición lateral:** k-WTA por tipo de nodo, para que compitan los recuerdos rivales.
4. **Efecto abanico:** los nodos con muchas aristas reparten menos activación a cada una.
5. **Resultado:** el subgrafo activo (y no una lista top-k) se ofrece al workspace global. Laya, en su papel de hipocampo, decide si entra.

## 5. Laya como puerta (hipocampo + prefrontal)

Laya responde preguntas tipadas sobre el estado y la SAMN, siempre con una probabilidad:

| Pregunta | Tipo | Función |
|---|---|---|
| ¿Codificar este tramo como episodio? | booleano | Novedad, saliencia y valencia |
| ¿Qué recuerdo activo entra al workspace? | choice | Puerta de recuperación |
| ¿Consolidar este grupo de episodios en un concepto? | booleano | Sueño NREM |
| ¿Esta acción es válida? | booleano | Prefrontal y escalera de autonomía |
| ¿Esto lo percibí o lo imaginé/soñé? | choice | Monitor de realidad (HOT-2) |

## 6. Sueño: consolidación, recombinación y olvido

- **NREM:**
  - replay de episodios priorizados por `recompensa × sorpresa × |valencia| × recencia`, con estilo IER/SER;
  - detección de comunidades en el subgrafo episódico, que se convierten en nodos `Concept`;
  - las secuencias de acciones repetidas con éxito se convierten en nodos `Skill`.
- **REM:**
  - recombinación: si existen A→B y B→C, se propone la hipótesis A→C, marcada como `dreamed`, que se pondrá a prueba despierto;
  - contrafactuales: revivir un fallo cambiando solo el tramo crítico;
  - aquí actúa EGGROLL.
- **Olvido:** poda de aristas con `w < ε` y nodos con `B` bajo que no sostienen ningún concepto. Nada se borra de golpe; primero se degrada.

## 7. Lenguaje anclado

Las palabras (nodos `Word`) se enlazan con conceptos mediante la coocurrencia **palabra ↔ estado neuronal ↔ evento**. Cuando dices "guardar" y aparece el diálogo, se refuerza `Word(guardar) —grounds→ Concept(guardar)`.

El cerebro-RWKV habla activando palabras cuyos conceptos están activos. Así su lenguaje nace de su experiencia, no de texto ajeno.

## 8. SAMN dirige la evolución (MEM-EGGROLL)

- **Máscara:** los módulos y tipos celulares asociados al subgrafo del problema actual reciben más perturbación: `Δθ = M_SAMN ⊙ ABᵀ`.
- **Rango:** `r = g(incertidumbre, novedad, conflicto entre recuerdos activos)`.
- **Reencarnación:** cada política nueva hereda la SAMN, no solo los pesos.

## 9. Por qué importa para IIT (y lo que no demuestra)

La SAMN hace que el sistema sea **causalmente integrado y recurrente** a nivel de memoria. Todo recuerdo activo influye y es influido por el resto, en lugar de ser una base de datos pasiva.

- **Medible:** Φ (PyPhi, IIT 4.0) en subgrafos pequeños de la SAMN acoplados al conectoma, comparados con un grafo barajado y con la SAMN desconectada.
- **Límite honesto:** IIT sitúa Φ en el sustrato físico. Una SAMN integrada demuestra integración **del modelo causal**, que es el argumento funcionalista (Kanai & Ma), pero no obliga a IIT a aceptarla. Por eso se combina con el experimento en hardware analógico (BrainScaleS-2).

## 10. Implementación

- **Lenguaje y ubicación:** Rust, dentro de `edid` (módulo `samn`).
- **Grafo y persistencia:** grafo en memoria con índices por tipo; persistencia en redb como event log + snapshots.
- **Búsqueda por similitud:** HNSW sobre `emb`.
- **Coste objetivo:** la recuperación en menos de 10 ms con 10⁵ nodos en CPU. El sueño va por lotes.
- **Tests:**
  1. el recuerdo mejora con la repetición espaciada y empeora sin ella;
  2. hay interferencia y efecto abanico medibles;
  3. los conceptos consolidados generalizan a episodios nuevos;
  4. el monitor de realidad distingue `measured` de `dreamed`;
  5. aprender con SAMN supera a aprender con memoria plana (top-k) y sin memoria.

## 11. Parámetros iniciales (hipótesis a evolucionar)

| Parámetro | Valor inicial | Evoluciona |
|---|---|---|
| η (hebbiano) / λ (olvido) | 0,05 / 0,001 por paso | Sí |
| d (decaimiento ACT-R) | 0,5 | Sí |
| δ (decaimiento de activación) / T (pasos) | 0,2 / 5 | Sí |
| k (k-WTA) | 7 por tipo | Sí |
| τ (ventana causal) | 2 s | Sí |
