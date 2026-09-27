"""Fase 0: reproducir Shiu et al. 2024 (azúcar -> MN9) en CPU y medir velocidad.

Uso: .venv/bin/python experiments/phase0/sugar_mn9.py [630|783] [n_run]
"""
import json, sys, time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SHIU = ROOT / "external/fly/Drosophila_brain_model"
sys.path.insert(0, str(SHIU))
from model import run_exp, default_params as params  # noqa: E402
import utils as utl  # noqa: E402

VERSION = sys.argv[1] if len(sys.argv) > 1 else "630"
N_RUN = int(sys.argv[2]) if len(sys.argv) > 2 else 5
DATA = {
    "630": ("2023_03_23_completeness_630_final.csv", "2023_03_23_connectivity_630_final.parquet"),
    "783": ("Completeness_783.csv", "Connectivity_783.parquet"),
}[VERSION]
# IDs v630 del notebook oficial (example.ipynb): 21 GRN de azúcar (hemisferio derecho) y MN9
SUGAR_630 = [720575940624963786, 720575940630233916, 720575940637568838, 720575940638202345,
             720575940617000768, 720575940630797113, 720575940632889389, 720575940621754367,
             720575940621502051, 720575940640649691, 720575940639332736, 720575940616885538,
             720575940639198653, 720575940620900446, 720575940617937543, 720575940632425919,
             720575940633143833, 720575940612670570, 720575940628853239, 720575940629176663,
             720575940611875570]
MN9_630 = 720575940660219265
ids_file = ROOT / f"experiments/phase0/ids_{VERSION}.json"
if VERSION == "630":
    sugar, mn9 = SUGAR_630, MN9_630
else:
    ids = json.loads(ids_file.read_text())
    sugar, mn9 = ids["sugar"], ids["mn9"]

out = ROOT / f"experiments/phase0/results_{VERSION}"
params["n_run"] = N_RUN
out.mkdir(parents=True, exist_ok=True)
t0 = time.time()
run_exp(exp_name="sugarR", neu_exc=sugar, path_res=str(out), path_comp=str(SHIU / DATA[0]),
        path_con=str(SHIU / DATA[1]), params=params, n_proc=-1)
wall = time.time() - t0
df = utl.load_exps([str(out / "sugarR.parquet")])
rate, _ = utl.get_rate(df, t_run=params["t_run"], n_run=N_RUN)
mn9_rate = float(rate["sugarR"].get(mn9, 0.0))
summary = {"version": VERSION, "n_run": N_RUN, "t_run_s": 1.0,
           "wall_s": round(wall, 1), "wall_per_bio_s": round(wall / N_RUN, 1),
           "mn9_rate_hz": mn9_rate, "n_active_neurons": int((rate["sugarR"] > 0).sum()),
           "top10": {str(k): float(v) for k, v in rate["sugarR"].sort_values(ascending=False).head(10).items()}}
(out / "summary.json").write_text(json.dumps(summary, indent=1))
print(json.dumps(summary, indent=1))
