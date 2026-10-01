"""Exporta la morfología real de las neuronas de FlyWire v783 para el cerebro 3D de EDI.

Fuente: Schlegel et al., Nature 2024, suplemento (Zenodo 10877326, CC-BY-4.0):
`sk_lod1_783_healed_ds2.parquet` — esqueleto (node_id, parent_id, radius, x, y, z en nm) de cada neurona.

Simplificación honesta: se conservan TODOS los puntos de ramificación, puntas y raíz, y además
los nodos con node_id múltiplo de `STRIDE`; cada nodo conservado se une a su ancestro
conservado más cercano. La topología y el recorrido son reales; solo se reduce la densidad de vértices.

Salida (data/brain/flywire783/):
  skel_<lod>.i16   segmentos: 2 vértices × (x,y,z) int16, en el mismo marco normalizado que los somas
  skel_<lod>.u32   índice de neurona (orden de Shiu) por segmento
  skel.json        escala de cuantización y estadísticas
"""
import json, sys
import numpy as np
import pyarrow.parquet as pq
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "data/skeletons/sk_lod1_783_healed_ds2.parquet"
OUT = ROOT / "data/brain/flywire783"
LODS = {"full": 12, "small": 48}     # paso a lo largo de cada rama
Q = 26000.0                           # int16 = coord_normalizada * Q

ids = np.fromfile(OUT / "root_id.i64", dtype=np.int64)
id2i = {int(r): i for i, r in enumerate(ids)}
# mismo marco que edid::positions(): centro y semiescala de las posiciones de soma
pos = np.fromfile(OUT / "pos.f32", dtype=np.float32).reshape(-1, 3)
ok = np.isfinite(pos[:, 0])
lo, hi = pos[ok].min(0), pos[ok].max(0)
ctr = (lo + hi) / 2
sc = float(max((hi - lo).max(), 1.0) / 2)

def segments(ni, node, parent, xyz, stride):
    """Vectorizado sobre un bloque de neuronas completas. Devuelve (seg[n,6], neurona[n])."""
    key = (ni.astype(np.int64) << 32) | node.astype(np.int64)
    order = np.argsort(key, kind="stable")
    skey = key[order]
    has_p = parent >= 0
    pkey = (ni.astype(np.int64) << 32) | np.where(has_p, parent, 0).astype(np.int64)
    pos_ = np.searchsorted(skey, pkey)
    pos_ = np.clip(pos_, 0, len(skey) - 1)
    prow = np.where(has_p & (skey[pos_] == pkey), order[pos_], -1)
    nchild = np.bincount(prow[prow >= 0], minlength=len(node))
    keep = (prow < 0) | (nchild != 1) | (node % stride == 0)
    anc = prow.copy()
    for _ in range(4 * stride + 50):
        m = (anc >= 0) & ~keep[np.maximum(anc, 0)]
        if not m.any():
            break
        anc[m] = prow[anc[m]]
    sel = np.flatnonzero(keep & (anc >= 0))
    seg = np.concatenate([xyz[sel], xyz[anc[sel]]], 1)
    return seg, ni[sel].astype(np.uint32)

def main():
    import pandas as pd
    f = pq.ParquetFile(SRC)
    index = pd.Index(ids)
    out = {lod: ([], []) for lod in LODS}
    stats = {"rows": 0, "neurons": 0, "unmatched_neurons": 0}
    carry = None
    def flush(t):
        ni = index.get_indexer(t["neuron"])
        ok_ = ni >= 0
        stats["unmatched_neurons"] += int(len(np.unique(t["neuron"][~ok_])))
        stats["neurons"] += int(len(np.unique(ni[ok_])))
        xyz = ((np.stack([t["x"], t["y"], t["z"]], 1)[ok_] - ctr) / sc) * np.array([1, -1, 1], np.float32)
        for lod, stride in LODS.items():
            seg, nid = segments(ni[ok_], t["node_id"][ok_], t["parent_id"][ok_], xyz.astype(np.float32), stride)
            out[lod][0].append(np.clip(seg * Q, -32767, 32767).astype(np.int16))
            out[lod][1].append(nid)
    cols = ["node_id", "parent_id", "x", "y", "z", "neuron"]
    for b in f.iter_batches(batch_size=6_000_000, columns=cols):
        t = {c: b.column(c).to_numpy() for c in cols}
        if carry is not None:
            t = {c: np.concatenate([carry[c], t[c]]) for c in cols}
        last = t["neuron"][-1]
        cut = len(t["neuron"]) - int(np.argmax(t["neuron"][::-1] != last)) if (t["neuron"] != last).any() else 0
        carry = {c: t[c][cut:] for c in cols}
        if cut:
            flush({c: t[c][:cut] for c in cols})
        stats["rows"] += int(b.num_rows)
        print(f"filas {stats['rows']:,} · neuronas {stats['neurons']:,}", file=sys.stderr, flush=True)
    if carry is not None and len(carry["neuron"]):
        flush(carry)
    meta = {"source": "Schlegel et al. 2024, Zenodo 10877326 (CC-BY-4.0), sk_lod1_783_healed_ds2",
            "quant": Q, "frame": "igual que pos.f32 normalizado por edid (centro/semiescala de somas, y invertida)",
            "stride": LODS, **stats}
    for lod in LODS:
        seg = np.concatenate(out[lod][0]); nid = np.concatenate(out[lod][1])
        seg.tofile(OUT / f"skel_{lod}.i16"); nid.tofile(OUT / f"skel_{lod}.u32")
        meta[f"segments_{lod}"] = int(len(nid))
    (OUT / "skel.json").write_text(json.dumps(meta, indent=1))
    print(json.dumps(meta, indent=1))

if __name__ == "__main__":
    main()
