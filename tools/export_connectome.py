"""Exporta el conectoma FlyWire v783 (orden de Shiu et al.) a binarios planos para el motor Rust.

Salida en data/brain/flywire783/:
  meta.json          n, n_edges, grupos de neuronas (índices)
  row_ptr.u32        CSR por neurona presináptica (n+1)
  col.u32            índice postsináptico por arista
  w.i32              nº sinapsis con signo (Excitatory x Connectivity)
  root_id.i64        root_id FlyWire por índice
  pos.f32            posición (x,y,z) por índice, en nm (NaN si falta)
"""
import json
import numpy as np
import pandas as pd
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHIU = ROOT / "external/fly/Drosophila_brain_model"
ANN = ROOT / "external/fly/flywire_annotations/supplemental_files/Supplemental_file1_neuron_annotations.tsv"
OUT = ROOT / "data/brain/flywire783"
OUT.mkdir(parents=True, exist_ok=True)

comp = pd.read_csv(SHIU / "Completeness_783.csv", index_col=0)
ids = comp.index.values.astype(np.int64)
n = len(ids)
con = pd.read_parquet(SHIU / "Connectivity_783.parquet",
                      columns=["Presynaptic_Index", "Postsynaptic_Index", "Excitatory x Connectivity"])
con = con.sort_values(["Presynaptic_Index", "Postsynaptic_Index"], kind="stable")
pre = con["Presynaptic_Index"].values.astype(np.int64)
row_ptr = np.zeros(n + 1, dtype=np.uint32)
np.add.at(row_ptr, pre + 1, 1)
row_ptr = np.cumsum(row_ptr, dtype=np.uint64).astype(np.uint32)
row_ptr.tofile(OUT / "row_ptr.u32")
con["Postsynaptic_Index"].values.astype(np.uint32).tofile(OUT / "col.u32")
con["Excitatory x Connectivity"].values.astype(np.int32).tofile(OUT / "w.i32")
ids.tofile(OUT / "root_id.i64")

ann = pd.read_csv(ANN, sep="\t", low_memory=False).set_index("root_id")
ann = ann[~ann.index.duplicated()]
a = ann.reindex(ids)
pos = a[["pos_x", "pos_y", "pos_z"]].to_numpy(dtype=np.float32) * np.array([4, 4, 40], np.float32)
pos.tofile(OUT / "pos.f32")

def idx(mask):
    return [int(i) for i in np.flatnonzero(mask.to_numpy())]

sub = a.cell_sub_class.astype(str)
groups = {
    "auditory_jo": idx(sub.eq("auditory")),
    "descending": idx(a.super_class.eq("descending")),
    "motor": idx(a.super_class.eq("motor")),
    "sugar_grn": idx(sub.eq("sugar/water")),
    "photoreceptor": idx(a.cell_class.astype(str).eq("photoreceptors") if "photoreceptors" in set(a.cell_class.astype(str)) else a.cell_type.astype(str).str.match(r"^R[1-8]")),
    "kenyon_cell": idx(a.cell_class.astype(str).eq("Kenyon_Cell")),
    "mbon": idx(a.cell_class.astype(str).eq("MBON")),
    "dan": idx(a.cell_class.astype(str).eq("DAN")),
}
id2i = {int(r): i for i, r in enumerate(ids)}
groups["mn9"] = [id2i[720575940660219265]]
sugar = json.loads((ROOT / "experiments/phase0/ids_783.json").read_text())["sugar"]
groups["sugar_shiu"] = [id2i[s] for s in sugar]
meta = {"source": "FlyWire v783 via Shiu et al. Connectivity_783.parquet", "n": n, "n_edges": int(len(con)),
        "groups": groups, "group_sizes": {k: len(v) for k, v in groups.items()}}
(OUT / "meta.json").write_text(json.dumps(meta))
print(json.dumps({k: meta[k] for k in ("n", "n_edges")}), meta["group_sizes"])
