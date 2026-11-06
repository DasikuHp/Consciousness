# Cómo se habla con EDI (entrada, lógica y personaje)

## Entrada
Se le habla desde el **widget**, desde la **ventana del cerebro** (dashboard) o con el **comando `edi`**:
- **Texto:** escribes y pulsas Enter. ↑/↓ recorren lo que ya has dicho.
- **Voz:** 🎤 dicta con el reconocimiento de voz del navegador (Edge/Chrome; puede usar su servicio en la nube). 🔊 hace que EDI diga sus respuestas en voz alta (síntesis local del sistema).
- **En cada respuesta suya:** 👍 / 👎 (dopamina), ✏️ para enseñarle qué debía decir, y ? para ver por qué lo dijo (qué olió y con qué valencia).
- **Atajos en la caja:** `+` / `-` premian o castigan su última respuesta; `= texto` le enseña. `/` abre las órdenes (Tab completa); en el dashboard, `/` desde cualquier sitio enfoca la caja.
- **Terminal:**
  - `edi hablar`: conversación en la terminal (también órdenes, `+`, `-`, `= texto`)
  - `edi say hola`, `edi +`, `edi -`, `edi teach hola que tal`
  - `edi chat 12`, `edi status`, `edi diario`, `edi nota …`, `edi recuerdame 10m …`
  - Windows: `edi.cmd …` (las mismas; si el texto lleva `=>`, entre comillas)

## Órdenes (usos)
Empiezan por `/` en el chat (o `edi ORDEN` en la terminal). **Las contesta el sistema, no el cerebro**: salen como turnos `sistema`, aparte de lo que dice EDI. Lo que sí entra al cerebro es lo percibido (notas, comandos, puertos), igual que el resto de sus sentidos.

| Orden | Qué hace |
|---|---|
| `/diario [horas]` | Lo que percibió: tiempo aproximado por app en primer plano (de un cambio de ventana al siguiente, máx. 30 min), comandos de la terminal y cuántos fallaron, puertos nuevos, conversación (frases, 👍/👎, enseñanzas, notas), lo que más le dijiste y sus siestas |
| `/nota TEXTO` · `/notas [palabra]` | Notas en `notes.jsonl`. La nota también la *oye*: sus palabras entran al oído y a la memoria (luego `/recuerda leche` la asocia) |
| `/recuerdame 10m TEXTO` | Recordatorio: `10m`, `2h`, `1h30m`, `90s`, `45` (min), `en 20 minutos`, `18:30` / `a las 18:30` (hoy o mañana). Persiste (`reminders.json`); si EDI estaba apagada, avisa al despertar «con retraso» |
| `/recordatorios` · `/borra N` | Pendientes y cancelar |
| `/recuerda PALABRA` | Qué asocia (SAMN), qué vino después la última vez (ROSA) y su valencia en las sinapsis KC→MBON |
| `/enseña PREGUNTA => RESPUESTA` | Le enseña un par sin esperar a que lo diga (refuerzo con dopamina incluido). En lote: `edi teach-file frases.txt` (una línea `pregunta => respuesta` por par) |
| `/aprendido` · `/estado` · `/duerme` · `/ayuda` | Progreso, estado vital, siesta forzada, lista de órdenes |

**Avisos:** cuando vence un recordatorio o se abre un puerto que **nunca** había visto escuchar (con el programa que lo abrió: `/proc` en Linux, `netstat` en Windows), sale un turno `aviso`/`alerta`, la cara reacciona, el widget late y llega una notificación de escritorio (`notify-send` en Linux; toast de Windows vía PowerShell). `EDI_NOTIFY=0` las apaga. La primera vez que arranca, los puertos abiertos se apuntan en silencio (`ports.json`).

## Sentido nuevo: tu terminal
`eval "$(edi hook bash)"` en `~/.bashrc` (o `edi hook zsh` en `~/.zshrc`; en Windows `edi.cmd hook` lo añade a tu perfil de PowerShell). Tras cada comando envía **solo su nombre** (`git`, `cargo`; salta `sudo`), nunca los argumentos, y si falló. Entra como evento `CMD:git` (valencia +0,05) o `FALLO:git` (−0,3): memoria, olor y predicción ROSA como cualquier otro sentido, y alimenta el diario. La cara comenta de vez en cuando (sin tapar la conversación).

## API local (127.0.0.1:7077)
| | |
|---|---|
| `POST /say` (texto) | crea tu turno. Con `?wait=0` responde al momento; si no, espera la respuesta de EDI (≤ 10 s). Si empieza por `/` es una orden. `?fmt=text`: respuesta en texto plano (terminal) |
| `POST /sense` (`CMD:git`, `APP:…`, `EV:…`; una por línea) | sentidos externos; `?ok=0` = el comando falló. Solo etiquetas de ≤ 48 caracteres |
| `GET /diario?h=24` | el diario en texto |
| `GET /chat?since=ID` | turnos nuevos (tuyos y suyos, con tipo, candidatas y valencias), estado reciente (👍/👎) y `thinking` (cuántas posibles respuestas lleva olidas) |
| `POST /reward?turn=ID` (`+`/`-`) | dopamina sobre esa respuesta, aunque llegue tarde. Sin `turn`, va a la última |
| `POST /teach?turn=ID` (texto) | «a eso se responde: texto». Con `pregunta => respuesta` (una o varias líneas, hasta 500) enseña pares sin turno |
| `GET /recall?q=…` | recuerdos asociados (acepta tildes) |

## Lógica de una frase tuya
1. **La oye.** La frase entra al oído (órgano de Johnston) y cada palabra se huele como un concepto. ROSA la compara con lo que esperaba y SAMN la guarda en la memoria episódica.
2. **Distingue la aprobación de la conversación.** Una frase hecha solo de aprobación o desaprobación («muy bien», «mal», «perfecto») no se contesta: es dopamina sobre su última respuesta. «estoy bien» o «gracias» sí son conversación.
3. **Imita.** Si le contestas a algo que EDI acaba de decir (en menos de 2 min), tu frase queda como una respuesta humana a eso. Así aprende cómo se contesta a «hola que tal». Nunca se copia a sí misma.
4. **Responde.**
   - Las candidatas son lo que un humano contestó tras ese contexto (al menos «palabra + fin de turno»): **imitación**.
   - Si esa frase exacta no la ha oído nunca, busca frases **parecidas**: las que comparten palabras con la tuya, cada palabra pesada por lo rara que es (idf; «gracias» pesa más que «que») y dividido por √longitud. Candidatas = las respuestas humanas a esas frases, y la frase se imita entera: **asociación** («muchas gracias» → «de nada» porque ya oyó «gracias → de nada»).
   - Si no sabe nada, hace ecolalia: repite la palabra más nueva para ella, como un bebé.
   - Huele cada candidata junto al contexto (las células de Kenyon codifican la conjunción) y elige la de mejor valencia en sus sinapsis KC→MBON, con un 10 % de exploración.
   - Completa la frase imitando cómo la dijo el humano.
5. **Aprende de tu feedback.**
   - 👍 / 👎 reactivan (replay) el olor de lo que dijo mientras llega la dopamina: PAM para el premio, PPL1 para el castigo.
   - ✏️ castiga lo que dijo, refuerza la respuesta correcta y la guarda para imitarla.
6. **Recuerda.** La conversación persiste en `chat.jsonl` (cada cambio de un turno, como su 👍/👎, se añade con el mismo id y al cargar gana el último) y continúa tras reiniciar.

Lo enseñado o premiado desde fuera (botones, `/enseña`, `edi teach`) se procesa **antes** de oír las frases nuevas del mismo instante: «enseña» seguido de «pregunta» funciona aunque lleguen juntos.

**Medido en vivo** (3 frases, profesor que premia o enseña):
- Ronda 1: ecolalia.
- Rondas 2–6: **3/3 estable**.

Antes del arreglo de la imitación oscilaba entre 0 y 3/3, porque tomaba tu frase siguiente como si fuera la respuesta a la anterior.

**Medido en vivo (asociación, 2026-11-06):** se le enseñan 8 pares (`hola => hola que tal`, `gracias => de nada`…) y se le pregunta con 8 paráfrasis que nunca oyó («oye como estas hoy», «te gusta la musica clasica», «buenas noches edi»…).
- Solo con la regla de sufijo exacto, 3/8 tenían coincidencia; el resto, ecolalia.
- Con asociación (primera versión): 5/8. Fallaba porque el cuerpo fungiforme elegía entre candidatas de parecido muy desigual (la valencia de una frase nueva es casi ruido) y porque podía proponer tu propia frase.
- Con la puerta de parecido (≥ 60 % de la mejor) y sin eco de tu frase: **8/8 en 3 repeticiones desde cero**.

Límite honesto: el parecido es de palabras, no de significado («¿cómo te va?» no se parece a «¿cómo estás?» salvo por «cómo»).

## Personaje
El personaje vectorial está en `os/assets/face/faces.json` y se genera con `tools/edi_face.py`:
- **Cuerpo:** geometría exacta medida sobre la media de los 30 diseños (arco R=100/75, error < 0,5 px).
- **Rasgos:** primitivas (media elipse, arco, línea, elipse, punto, cuadrilátero, chevrón) ajustadas a cada diseño por `tools/fit_faces.py`, con un único trazo de 7,06.
- **Animación:** los grupos `#ojo-i`, `#ojo-d` y `#boca` permiten parpadear y mover la boca mientras habla.
