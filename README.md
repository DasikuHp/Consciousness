# Consciousness

Conectomas reales (mosca, con datos humanos y de otros animales) ejecutados como redes spiking, acoplados a **Laya** como "córtex prefrontal" determinista, para medir marcadores asociados a la conciencia y a la computación neuronal. Objetivo y reglas: [`goal.md`](goal.md). Catálogo investigado de recursos: [`docs/research/catalogo_recursos.md`](docs/research/catalogo_recursos.md).

## Qué contiene el repo

| Carpeta | Contenido |
|---|---|
| `vault/` | **Datos y pesos dentro de git**, troceados en partes de <95 MB con `sha256` (`_index.json` por recurso) |
| `external/` | Código de terceros como **submódulos git** fijados a un commit (`fly/`, `sim/`, `consciousness/`, `lang/`, `other/`) |
| `manifest.json` | Lista de todo recurso externo: URL, licencia y si va en el repo (`in_repo`) o solo por script |
| `tools/vault.py` | Descargar / trocear / reensamblar |
| `experiments/` | Experimentos (fase 0: Shiu et al., azúcar → MN9) |

### Dentro del repo (`vault/`)
- **Laya** completo: inglés, multilingüe y typed-decisions (2,37 GB, Apache-2.0)
- **RWKV-7 G1d 0,1B** (0,38 GB, Apache-2.0)
- **MaleCNS v1.0**: pesos de conectividad neurona→neurona, anotaciones, neurotransmisores (CC-BY-4.0)
- **BANC v888**: metadatos, edgelist v3, NT, somas, puntos representativos, mallas de neuropilos, neuropéptidos, ground truth de NT, muestra de revisión manual de sinapsis, registros y conducta (CC-BY-4.0)
- **FlyWire v783**: IDs revisados y recuento por neuropilo (el conectoma que usa el modelo LIF viene en `external/fly/Drosophila_brain_model`)
- **Humano**: H01, 104 neuronas revisadas en SWC (CC-BY-4.0); `tvb_data`, con conectomas humano, macaco y ratón para The Virtual Brain (GPL-3.0)

### Solo por script (demasiado grande para GitHub o para este disco)
Sinapsis individuales de FlyWire (9,5 GB), de MaleCNS (~20 GB) y de BANC (~20 GB); esqueletos; NBLAST; RWKV-7 0,4B. BANC completo pesa 536 GB y H01 completo 1,4 PB.

## Uso

```bash
git clone --recurse-submodules --shallow-submodules <repo>
python3 -m venv .venv && .venv/bin/pip install "numpy<2.4" brian2 pandas pyarrow joblib huggingface_hub
.venv/bin/python tools/vault.py unpack            # vault/ -> data/ (verifica sha256)
.venv/bin/python tools/vault.py fetch --all       # además, lo que no está en el repo
.venv/bin/python experiments/phase0/sugar_mn9.py 630 5
```

Brian2 2.9 no funciona con NumPy ≥ 2.4 (`ndarray.ptp` eliminado), así que hay que fijar `numpy<2.4`.

## Licencias
Código propio: por decidir (ver `goal.md` §15). Cada dato y submódulo conserva su licencia (ver `manifest.json` y el catálogo). Varios submódulos son **GPL** (PCIst, PyPhi, NEST, Nengo, navis, fly-brain de Eon); están como submódulos o dependencias y no se ha copiado su código.
