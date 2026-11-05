# Cómo se habla con EDI (entrada, lógica y personaje)

## Entrada
Se le habla desde el **widget**, desde la **ventana del cerebro** (dashboard) o con el **comando `edi`**:
- **Texto:** escribes y pulsas Enter. ↑/↓ recorren lo que ya has dicho.
- **Voz:** 🎤 dicta con el reconocimiento de voz del navegador (Edge/Chrome; puede usar su servicio en la nube). 🔊 hace que EDI diga sus respuestas en voz alta (síntesis local del sistema).
- **En cada respuesta suya:** 👍 / 👎 (dopamina), ✏️ para enseñarle qué debía decir, y ? para ver por qué lo dijo (qué olió y con qué valencia).
- **Atajos en la caja:** `+` / `-` premian o castigan su última respuesta; `= texto` le enseña.
- **Terminal:**
  - `edi say hola`, `edi +`, `edi -`
  - `edi teach hola que tal`
  - `edi chat 12`, `edi status`
  - Windows: `edi.cmd …`

## API local (127.0.0.1:7077)
| | |
|---|---|
| `POST /say` (texto) | crea tu turno. Con `?wait=0` responde al momento; si no, espera la respuesta de EDI (≤ 10 s) |
| `GET /chat?since=ID` | turnos nuevos (tuyos y suyos, con tipo, candidatas y valencias), estado reciente (👍/👎) y `thinking` (cuántas posibles respuestas lleva olidas) |
| `POST /reward?turn=ID` (`+`/`-`) | dopamina sobre esa respuesta, aunque llegue tarde. Sin `turn`, va a la última |
| `POST /teach?turn=ID` (texto) | «a eso se responde: texto» |
| `GET /recall?q=…` | recuerdos asociados (acepta tildes) |

## Lógica de una frase tuya
1. **La oye.** La frase entra al oído (órgano de Johnston) y cada palabra se huele como un concepto. ROSA la compara con lo que esperaba y SAMN la guarda en la memoria episódica.
2. **Distingue la aprobación de la conversación.** Una frase hecha solo de aprobación o desaprobación («muy bien», «mal», «perfecto») no se contesta: es dopamina sobre su última respuesta. «estoy bien» o «gracias» sí son conversación.
3. **Imita.** Si le contestas a algo que EDI acaba de decir (en menos de 2 min), tu frase queda como una respuesta humana a eso. Así aprende cómo se contesta a «hola que tal». Nunca se copia a sí misma.
4. **Responde.**
   - Las candidatas son lo que un humano contestó tras ese contexto (al menos «palabra + fin de turno»).
   - Si no sabe nada, hace ecolalia: repite la palabra más nueva para ella, como un bebé.
   - Huele cada candidata junto al contexto (las células de Kenyon codifican la conjunción) y elige la de mejor valencia en sus sinapsis KC→MBON, con un 10 % de exploración.
   - Completa la frase imitando cómo la dijo el humano.
5. **Aprende de tu feedback.**
   - 👍 / 👎 reactivan (replay) el olor de lo que dijo mientras llega la dopamina: PAM para el premio, PPL1 para el castigo.
   - ✏️ castiga lo que dijo, refuerza la respuesta correcta y la guarda para imitarla.
6. **Recuerda.** La conversación persiste en `chat.jsonl` y continúa tras reiniciar.

**Medido en vivo** (3 frases, profesor que premia o enseña):
- Ronda 1: ecolalia.
- Rondas 2–6: **3/3 estable**.

Antes del arreglo de la imitación oscilaba entre 0 y 3/3, porque tomaba tu frase siguiente como si fuera la respuesta a la anterior.

## Personaje
El personaje vectorial está en `os/assets/face/faces.json` y se genera con `tools/edi_face.py`:
- **Cuerpo:** geometría exacta medida sobre la media de los 30 diseños (arco R=100/75, error < 0,5 px).
- **Rasgos:** primitivas (media elipse, arco, línea, elipse, punto, cuadrilátero, chevrón) ajustadas a cada diseño por `tools/fit_faces.py`, con un único trazo de 7,06.
- **Animación:** los grupos `#ojo-i`, `#ojo-d` y `#boca` permiten parpadear y mover la boca mientras habla.
