# Habla v2: imitación + elección por el cuerpo fungiforme (2026-10-07)

**Mecanismo** (`edi/crates/edid/src/speech.rs`):
1. **Imitación (mimetización humana).** ROSA guarda pares «lo que se dijo → lo que un humano respondió». Las candidatas para responder son esas respuestas humanas. La frase se completa copiando cómo la dijo el humano. Si no sabe nada, repite la última palabra (ecolalia, como un bebé). Nunca se copia a sí misma y no toma como respuesta las frases que son solo aprobación o desaprobación.
2. **Elección (centro de memoria).** Cada candidata se huele junto al contexto: 3 glomérulos de la frase oída y 3 de la candidata. Las células de Kenyon (KC) codifican esa conjunción y gana la de mejor valencia en sus sinapsis KC→MBON.
3. **Aprendizaje.** Tu "bien" o "mal" reactiva (replay) la conjunción que EDI acaba de decir mientras llega la dopamina: PAM si es premio, PPL1 si es castigo.

## Prueba controlada (`speech_test`, profesor con guion, 8 preguntas × 3 respuestas oídas)
El profesor solo aprueba la respuesta amable. EDI elige con un 10 % de exploración y completa la frase imitando. El azar es 33 %.

| condición | semilla 1 | semilla 2 | semilla 3 | media rondas 11–20 |
|---|---|---|---|---|
| conectoma real | 55 % | 68 % | 60 % | **61 %** |
| conectoma barajado | 45 % | 13 % | 50 % | 36 % |
| sin plasticidad (10 rondas) | 25 % | – | – | 25 % |

En las rondas 1–5, el real acierta un 38 %; sube a 61 % (máximo 8/8 en una ronda). El real supera al barajado en 3/3 semillas.

**Honesto:**
- El aprendizaje es lento: ~20 ensayos por pregunta para llegar al 61 %. Una mosca aprende un olor en 1–3 ensayos.
- El efecto existe, pero la variabilidad es alta.
- Las palabras vienen de la imitación (ROSA). El cerebro de la mosca solo elige entre lo imitado: no genera lenguaje.

## En vivo (`edid`, por HTTP)
Enseñando 3 frases (pregunta → EDI responde → "muy bien" o "mal" → el humano dice la correcta):
- Ronda 1: ecolalia («estas», «llamas»).
- Ronda 2: **3/3 imitando** («hola que tal», «bien gracias y tu», «me llamo edi»).
- Rondas 3–6: 1–2/3, porque aparecen más candidatas (otras frases de la conversación) y la elección por dopamina aún no converge en 6 rondas.
