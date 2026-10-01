"""Exporta la morfología real de las neuronas de FlyWire v783 para el cerebro 3D de EDI.

Fuente: Schlegel et al., Nature 2024, suplemento (Zenodo 10877326, CC-BY-4.0):
`sk_lod1_783_healed_ds2.parquet` — esqueleto (node_id, parent_id, radius, x, y, z en nm) de cada neurona.

Simplificación honesta: se conservan TODOS los puntos de ramificación, puntas y raíz, y además
un nodo de cada `STRIDE` a lo largo de cada rama; cada nodo conservado se une a su ancestro
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

def simplify(node, parent, stride):
    """Devuelve pares (hijo, ancestro) de nodos conservados."""
    idx = {int(n): k for k, n in enumerate(node)}
    par = np.array([idx.get(int(p), -1) for p in parent])
    nchild = np.bincount(par[par >= 0], minlength=len(node))
    keep = (par < 0) | (nchild != 1)
    # profundidad a lo largo del árbol para muestrear cada `stride`
    order = np.argsort(par >= 0, kind="stable")  # raíces primero
    depth = np.full(len(node), -1)
    stack = [k for k in range(len(node)) if par[k] < 0]
    children = [[] for _ in range(len(node))]
    for k, p in enumerate(par):
        if p >= 0:
            children[p].append(k)
    for r in stack:
        depth[r] = 0
    while stack:
        k = stack.pop()
        for ch in children[k]:
            depth[ch] = depth[k] + 1
            stack.append(ch)
    keep |= (depth % stride == 0)
    # ancestro conservado más cercano
    anc = np.full(len(node), -1)
    stack = [k for k in range(len(node)) if par[k] < 0]
    while stack:
        k = stack.pop()
        for ch in children[k]:
            anc[ch] = k if keep[k] else anc[k]
            stack.append(ch)
    kids = np.flatnonzero(keep & (anc >= 0))
    _ = order
    return kids, anc[kids]

def main():
    f = pq.ParquetFile(SRC)
    out = {lod: ([], []) for lod in LODS}
    carry = None
    stats = {"rows": 0, "neurons": 0}
    def flush(df):
        for nid, g in df.groupby("neuron", sort=False):
            i = id2i.get(int(nid))
            if i is None:
                continue
            xyz = ((g[["x", "y", "z"]].to_numpy(np.float32) - ctr) / sc) * np.array([1, -1, 1], np.float32)
            node, parent = g["node_id"].to_numpy(), g["parent_id"].to_numpy()
            for lod, stride in LODS.items():
                a, b = simplify(node, parent, stride)
                if len(a) == 0:
                    continue
                seg = np.stack([xyz[a], xyz[b]], 1).reshape(-1, 6)
                out[lod][0].append(np.clip(seg * Q, -32767, 32767).astype(np.int16))
                out[lod][1].append(np.full(len(a), i, np.uint32))
            stats["neurons"] += 1
    for rg in range(f.metadata.num_row_groups):
        df = f.read_row_group(rg).to_pandas()
        stats["rows"] += len(df)
        if carry is not None:
            import pandas as pd
            df = pd.concat([carry, df])
        last = df["neuron"].iloc[-1]
        carry = df[df["neuron"] == last]
        flush(df[df["neuron"] != last])
        print(f"grupo {rg + 1}/{f.metadata.num_row_groups} · neuronas {stats['neurons']}", file=sys.stderr, flush=True)
    if carry is not None:
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
