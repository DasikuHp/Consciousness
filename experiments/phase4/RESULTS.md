# Fase 4 — Connectome-RWKV v0 (600 gen. EGGROLL, CPU)

Tarea: predecir el siguiente carácter en un corpus español corto. Referencias: unigrama 2,916 · bigrama 2,357 · uniforme 3,64 (entropía cruzada, nats).

| Condición | CE final | Acierto | Qué dice |
|---|---|---|---|
| Conectoma real | 2,903 | 19,8 % | solo espacios (frecuencia de letras) |
| Conectoma barajado | 2,908 | 19,8 % | solo espacios |
| Sin cerebro (entrada → lectura directa) | **2,477** | **28,4 %** | "o    eo  e ..." |

**Resultado honesto:** en v0 el cerebro real no transmite la identidad de los caracteres del oído (JO) a las descendentes (DN) en ventanas de 10 ms. Aprende lo mismo que el barajado y menos que sin cerebro. EGGROLL funciona (el bypass baja del unigrama).

**Hipótesis para v1:**
- la vía JO→DN es demasiado larga o silenciosa: leer también de neuronas centrales activas (p. ej. AMMC/WED, cuerpo fungiforme) o de todas las activas;
- ventanas por carácter más largas (30–50 ms);
- entrada más fuerte o más neuronas de entrada;
- evolucionar ganancias por tipo celular;
- más generaciones.
