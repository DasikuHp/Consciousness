"""Vectoriza las 30 expresiones de EDI (docs/img/eddieexpresiones*.png) a SVG nítido.
Silueta (cuerpo blanco) + tinta (trazos negros), trazadas con potrace sobre el recorte ×4."""
from PIL import Image
import numpy as np, json, potrace
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "os/assets/face"
SHEETS = [("docs/img/eddieexpresiones2.png", 1), ("docs/img/eddieexpresiones.png", 11), ("docs/img/eddieexpresiones3.png", 21)]
idx = json.load(open(OUT / "index.json"))

def path_d(bm):
    out = []
    for c in potrace.Bitmap(~bm).trace(turdsize=8, alphamax=1.0, opticurve=True, opttolerance=0.2):
        s = c.start_point
        d = [f"M{s.x:.1f},{s.y:.1f}"]
        for g in c.segments:
            if g.is_corner:
                d.append(f"L{g.c.x:.1f},{g.c.y:.1f}L{g.end_point.x:.1f},{g.end_point.y:.1f}")
            else:
                d.append(f"C{g.c1.x:.1f},{g.c1.y:.1f} {g.c2.x:.1f},{g.c2.y:.1f} {g.end_point.x:.1f},{g.end_point.y:.1f}")
        out.append("".join(d) + "Z")
    return "".join(out)

def spans(mask, gap, minw):
    segs, x, n = [], 0, len(mask)
    while x < n:
        if mask[x]:
            s = x
            while x < n and mask[x]:
                x += 1
            segs.append((s, x))
        x += 1
    m = []
    for s, e in segs:
        if m and s - m[-1][1] < gap:
            m[-1] = (m[-1][0], e)
        else:
            m.append((s, e))
    return [q for q in m if q[1] - q[0] > minw]

for path, base in SHEETS:
    a = np.array(Image.open(ROOT / path).convert("L"))
    dark = a < 110
    for r, (y0, y1) in enumerate(spans(dark.sum(1) > 0, 1, 150)):
        for c, (x0, x1) in enumerate(spans(dark[y0:y1].sum(0) > 0, 40, 100)):
            crop = Image.fromarray(a[max(0, y0 - 8):y1 + 8, max(0, x0 - 8):x1 + 8])
            big = np.array(crop.resize((crop.width * 4, crop.height * 4), Image.BICUBIC))
            ink = big < 128
            h, w = ink.shape
            sil = np.zeros_like(ink)
            rmax = -1
            for i in range(h):
                xs = np.flatnonzero(ink[i])
                if len(xs):
                    rmax = max(rmax, xs[-1])
                    sil[i, xs[0]:rmax + 1] = True
            k = base + r * 5 + c
            svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">'
                   f'<path fill="#f6f6f2" d="{path_d(sil)}"/><path fill="#111" d="{path_d(ink)}"/></svg>')
            (OUT / idx[str(k)]["file"].replace(".png", ".svg")).write_text(svg)
print("ok")
