"""Ajusta las 30 expresiones de EDI a primitivas geométricas exactas (herramienta de desarrollo).

Para cada cara: separa los rasgos del cuerpo (cuerpo geométrico de edi_face.py), y ajusta cada
rasgo con su tipo de primitiva (indicado a mano abajo) minimizando la diferencia con el diseño
original a 4× (Powell). Un único grosor de trazo para todas las caras. Escribe
os/assets/face/faces.json, que edi_face.py convierte en SVG.
Requiere: numpy, pillow, scipy, shapely.   Uso: python3 tools/fit_faces.py
"""
import json, math, sys
import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage, optimize
from shapely.geometry import LinearRing, LineString, Point, Polygon, MultiPolygon
from shapely.ops import unary_union
sys.path.insert(0, str(__import__("pathlib").Path(__file__).parent))
import edi_face as E

S = 4  # supermuestreo
NAMES = json.loads((E.FACE / "index.json").read_text())
SHEETS = [("eddieexpresiones2.png", 1), ("eddieexpresiones.png", 11), ("eddieexpresiones3.png", 21)]
SPEC = {
    1: ["semi_d", "semi_d", "semi_d"], 2: ["semi_d", "semi_d", "line"], 3: ["semi_u", "semi_u", "ell"],
    4: ["semi_u", "semi_u", "arc_s"], 5: ["quad", "quad", "line"], 6: ["semi_u", "semi_u", "line"],
    7: ["semi_d", "semi_d", "arc_f"], 8: ["semi_u", "semi_u", "arc_f"], 9: ["semi_d", "semi_d", "semi_d"],
    10: ["arc_s", "arc_s", "ell"], 11: ["quadf+dot", "semi_d", "semi_d"], 12: ["semi_d", "semi_d", "line"],
    13: ["semi_u", "semi_u", "semi_d"], 14: ["semi_u", "semi_u", "semi_d"], 15: ["quad+dot", "quad+dot", "line"],
    16: ["semi_u", "semi_u", "semi_u"], 17: ["semi_d", "semi_d", "semi_u"], 18: ["semi_u", "semi_u", "semi_u"],
    19: ["semi_u", "semi_u", "semi_d"], 20: ["semi_u", "semi_u", "line"], 21: ["semi_u", "semi_u", "arc_s"],
    22: ["semi_d", "semi_d", "line"], 23: ["semi_u+dot", "semi_u+dot", "line"], 24: ["semi_d+bar", "semi_d+bar", "arc_f"],
    25: ["quad+dot", "quad+dot", "line"], 26: ["semi_u", "semi_u", "arc_s"], 27: ["ell+dot", "ell+dot+dot", "ell"],
    28: ["chev_r", "chev_l", "arc_f"], 29: ["semi_d", "semi_d+dot", "arc_s"], 30: ["line", "line", "semi_u"],
}


# ---------- geometría (shapely), coherente con prim_svg de edi_face ----------
def geom(p, w):
    t, a = p["t"], p["p"]
    if t == "semi":
        cx, cy, ra, rb, th = a
        P = E.rot(cx, cy, th)
        pts = [P(-ra, 0)] + [P(ra * math.cos(u), p["dir"] * rb * math.sin(u)) for u in np.linspace(0, math.pi, 48)]
        return LinearRing(pts).buffer(w / 2, join_style="mitre", mitre_limit=20)
    if t == "ell":
        cx, cy, ra, rb, th = a
        P = E.rot(cx, cy, th)
        return LinearRing([P(ra * math.cos(u), rb * math.sin(u)) for u in np.linspace(0, 2 * math.pi, 72)[:-1]]).buffer(w / 2)
    if t == "arc":
        cx, cy, ra, rb, th, t0, t1 = a
        P = E.rot(cx, cy, th)
        return LineString([P(ra * math.cos(math.radians(u)), rb * math.sin(math.radians(u))) for u in np.linspace(t0, t1, 40)]).buffer(w / 2, cap_style="flat")
    if t == "line":
        x0, y0, x1, y1, lw = a
        return LineString([(x0, y0), (x1, y1)]).buffer(max(lw, 1) / 2, cap_style="flat")
    if t == "dot":
        return Point(a[0], a[1]).buffer(max(a[2], 0.5))
    if t == "chev":
        return LineString([(a[0], a[1]), (a[2], a[3]), (a[4], a[5])]).buffer(w / 2, cap_style="flat", join_style="mitre", mitre_limit=20)
    if t == "quad":
        pts = [(a[2 * i], a[2 * i + 1]) for i in range(4)]
        rs = a[8:12] if len(a) >= 12 else [0] * 4
        ring = []
        for (pa, pb, r, sw), q in zip(E.fillet(pts, rs), pts):
            if r <= 0:
                ring.append(q); continue
            # centro del redondeo: desde el vértice por la bisectriz
            bx, by = (pa[0] + pb[0]) / 2 - q[0], (pa[1] + pb[1]) / 2 - q[1]
            lb = math.hypot(bx, by) or 1
            dq = math.hypot(pa[0] - q[0], pa[1] - q[1])
            cdist = math.hypot(dq, r)
            c = (q[0] + bx / lb * cdist, q[1] + by / lb * cdist)
            a0, a1 = math.atan2(pa[1] - c[1], pa[0] - c[0]), math.atan2(pb[1] - c[1], pb[0] - c[0])
            d = (a1 - a0 + math.pi) % (2 * math.pi) - math.pi
            ring += [(c[0] + r * math.cos(a0 + d * s), c[1] + r * math.sin(a0 + d * s)) for s in np.linspace(0, 1, 10)]
        if Polygon(ring).area < 1:
            return Point(pts[0]).buffer(0.1)
        return LinearRing(ring).buffer(w / 2, join_style="mitre", mitre_limit=20)
    raise ValueError(t)


def body_geom():
    m0, m1, mx = E.MID; b0, b1, bx = E.BASE
    ring = Point(E.CX, E.CY).buffer(E.RO, 128).difference(Point(E.CX, E.CY).buffer(E.RI, 128))
    top = ring.intersection(Polygon([(-5, -5), (205, -5), (205, E.CY), (-5, E.CY)]))
    rects = [Polygon([(0, E.CY), (E.T, E.CY), (E.T, b1), (0, b1)]), Polygon([(0, m0), (mx, m0), (mx, m1), (0, m1)]),
             Polygon([(0, b0), (bx, b0), (bx, b1), (0, b1)])]
    arms = LineString([(E.YTOP[0], m1), (E.CX, E.YJ), (E.YTOP[1], m1)]).buffer(E.YW / 2, cap_style="flat", join_style="mitre", mitre_limit=20)
    stem = LineString([(E.CX, E.YJ - 6), (E.CX, b0 + 2)]).buffer(E.YS / 2, cap_style="flat")
    return unary_union([top] + rects + [arms, stem])


def raster(g, win):
    """Máscara a 4× en la ventana win=(c0, r0, ancho, alto) de la imagen recortada."""
    c0, r0, W, H = win
    im = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(im)
    tf = lambda xy: [((x + 10) * S - c0, (y + 14) * S - r0) for x, y in xy]
    polys = g.geoms if isinstance(g, MultiPolygon) else [g]
    for pg in polys:
        if pg.is_empty or not hasattr(pg, "exterior"):
            continue
        d.polygon(tf(pg.exterior.coords), fill=1)
        for h in pg.interiors:
            d.polygon(tf(h.coords), fill=0)
    return np.asarray(im, dtype=np.float32)


# ---------- recortes alineados de las figuras ----------
def crops():
    R = E.ROOT / "docs/img"
    out = {}
    for f, base in SHEETS:
        a = 1 - np.asarray(Image.open(R / f).convert("L"), dtype=np.float32) / 255
        dark = a > 0.57
        rows = np.flatnonzero(dark.sum(1) > 0)
        bands = np.split(rows, np.flatnonzero(np.diff(rows) > 1) + 1)
        bands = [b for b in bands if len(b) > 150]
        for r, band in enumerate(bands):
            y0, y1 = band[0], band[-1] + 1
            cols = np.flatnonzero(dark[y0:y1].sum(0) > 0)
            groups = np.split(cols, np.flatnonzero(np.diff(cols) > 40) + 1)
            groups = [g for g in groups if len(g) > 100]
            for c, g in enumerate(groups):
                x0 = g[0]
                prof = a[y0 + 150:y0 + 180, x0 - 6:x0 + 40].mean(0)
                i = int(np.argmax(prof > 0.5)); lx = x0 - 6 + (i - 1) + (0.5 - prof[i - 1]) / (prof[i] - prof[i - 1])
                cl = a[y1 - 40:y1 + 8, x0 + 150:x0 + 180].mean(1)
                j = len(cl) - 1 - int(np.argmax(cl[::-1] > 0.5)); by = y1 - 40 + j + (cl[j] - 0.5) / (cl[j] - cl[j + 1])
                xs = lx - 10 + np.arange(225 * S) / S; ys = by - 325 + np.arange(335 * S) / S
                X, Y = np.meshgrid(xs, ys)
                out[base + r * 5 + c] = ndimage.map_coordinates(a, [Y, X], order=1)
    return out


# ---------- inicialización y ajuste ----------
def comp_xy(mask, win):
    r, c = np.nonzero(mask)
    return (c + win[0]) / S - 10, (r + win[1]) / S - 14  # coordenadas del diseño


def init(kind, x, y, w):
    x0, x1, y0, y1 = x.min(), x.max(), y.min(), y.max()
    xm, ym = (x0 + x1) / 2, (y0 + y1) / 2
    if kind in ("semi_d", "semi_u"):
        d = 1 if kind == "semi_d" else -1
        return {"t": "semi", "dir": d, "p": [xm, y0 + w / 2 if d > 0 else y1 - w / 2, (x1 - x0 - w) / 2, y1 - y0 - w, 0.0]}
    if kind == "ell":
        return {"t": "ell", "p": [xm, ym, (x1 - x0 - w) / 2, (y1 - y0 - w) / 2, 0.0]}
    if kind in ("arc_s", "arc_f"):
        r = (x1 - x0) / 2
        if kind == "arc_s":
            return {"t": "arc", "p": [xm, y1 - w / 2 - r, r, r, 0.0, 10.0, 170.0]}
        return {"t": "arc", "p": [xm, y0 + w / 2 + r, r, r, 0.0, 190.0, 350.0]}
    if kind == "line":
        P = np.c_[x, y]; m = P.mean(0); u = np.linalg.svd(P - m)[2][0]
        s = (P - m) @ u; n = (P - m) @ np.array([-u[1], u[0]])
        a, b = m + u * s.min(), m + u * s.max()
        return {"t": "line", "p": [a[0], a[1], b[0], b[1], float(n.max() - n.min())]}
    if kind in ("chev_r", "chev_l"):
        i = w / 2
        if kind == "chev_r":
            return {"t": "chev", "p": [x0 + i, y0 + i, x1 - i, ym, x0 + i, y1 - i]}
        return {"t": "chev", "p": [x1 - i, y0 + i, x0 + i, ym, x1 - i, y1 - i]}
    if kind in ("quad", "quadf"):
        P = np.c_[x, y]; c = P.mean(0)
        idx = [np.argmin(x + y), np.argmax(x - y), np.argmax(x + y), np.argmin(x - y)]
        pts = []
        for k in idx:
            v = P[k] - c; pts += list(P[k] - v / np.linalg.norm(v) * w * 0.7)
        return {"t": "quad", "p": pts + ([3.0] * 4 if kind == "quadf" else [])}
    raise ValueError(kind)


def fit_group(target, win, prims, w, free_w):
    """Ajuste conjunto de las primitivas de un rasgo (Powell sobre diferencia suavizada)."""
    T = ndimage.gaussian_filter(target, 2)
    sizes = [len(p["p"]) for p in prims]
    def unpack(v):
        out, i = [], 0
        for p, n in zip(prims, sizes):
            q = dict(p); q["p"] = list(v[i:i + n]); out.append(q); i += n
        return out, (v[i] if free_w else w)
    def loss(v):
        ps, ww = unpack(v)
        if ww < 2 or ww > 14:
            return 1e12
        try:
            g = unary_union([geom(p, ww) for p in ps])
        except Exception:
            return 1e12
        R = ndimage.gaussian_filter(raster(g, win), 2)
        return float(((R - T) ** 2).sum())
    v0 = [x for p in prims for x in p["p"]] + ([w] if free_w else [])
    best = optimize.minimize(loss, v0, method="Powell", options={"xtol": 0.05, "ftol": 1e-5, "maxfev": 2500})
    ps, ww = unpack(best.x)
    return ps, ww, best.fun


def residual_blobs(target, win, prims, w):
    R = raster(unary_union([geom(p, w) for p in prims]), win)
    res = (target > 0.5) & ~ndimage.binary_dilation(R > 0, iterations=3)
    lab, n = ndimage.label(res)
    blobs = []
    for i in range(1, n + 1):
        m = lab == i
        if m.sum() > 30:
            blobs.append(m)
    return sorted(blobs, key=lambda m: -m.sum())


def main():
    C = crops()
    W0 = 7.0
    win_full = (0, 0, 225 * S, 335 * S)
    body = raster(body_geom(), win_full)
    bodyd = ndimage.binary_dilation(body > 0, iterations=10)
    out, report, first = {}, [], {}
    only = [int(a) for a in sys.argv[1].split(",")] if len(sys.argv) > 1 else None
    for pass_ in (1, 2):
        widths = []
        for k in sorted(C):
            if only and k not in only:
                continue
            c = C[k]
            iou = ((c > 0.5) & (body > 0)).sum() / ((c > 0.5) & (bodyd) | (body > 0)).sum()
            feat = (c > 0.5) & ~bodyd
            lab, n = ndimage.label(feat)
            # rasgos: componentes por encima de la barra central (abajo solo quedan bordes del cuerpo)
            comps = [m for m in (lab == i for i in range(1, n + 1)) if m.sum() > 150 and np.nonzero(m)[0].mean() / S - 14 < E.MID[0] - 4]
            big = max(m.sum() for m in comps)
            comps = [m for m in comps if m.sum() > 0.08 * big]  # fuera astillas del borde del cuerpo
            cy = [np.nonzero(m)[0].mean() for m in comps]
            mouth = comps[int(np.argmax(cy))]
            eyes = sorted([m for m in comps if m is not mouth], key=lambda m: np.nonzero(m)[1].mean())
            if len(eyes) != 2:
                raise SystemExit(f"cara {k}: {len(comps)} componentes")
            groups, pivots = [], []
            for spec, m in zip(SPEC[k], eyes + [mouth]):
                kinds = spec.split("+")
                r, cc = np.nonzero(m)
                M = 40
                c0, r0 = max(cc.min() - M, 0), max(r.min() - M, 0)
                tgt = (c * ndimage.binary_dilation(m, iterations=4))[r0:r.max() + M, c0:cc.max() + M]
                win = (c0, r0, tgt.shape[1], tgt.shape[0])
                x, y = comp_xy(m[win[1]:win[1] + win[3], win[0]:win[0] + win[2]], win)
                w = W0 if pass_ == 1 else out["stroke"]
                gi = len(groups)
                if pass_ == 2:  # parte del pase 1 con el trazo común fijado
                    ps, _, fun = fit_group(tgt, win, first[k][gi], w, False)
                    groups.append([{**p, "p": [round(float(v), 2) for v in p["p"]]} for p in ps])
                    report.append((pass_, k, spec, round(fun, 1)))
                    pivots.append([round(float(x.mean()), 1), round(float(y.mean()), 1)])
                    continue
                prims = [init(kinds[0], x, y, w)]
                if prims[0]["t"] == "semi" and k in (7, 8, 17, 18, 22, 23, 24, 29) or prims[0]["t"] in ("arc",):
                    # multiarranque de giro para rasgos inclinados
                    cands = []
                    for th in (-20.0, 0.0, 20.0):
                        q = json.loads(json.dumps(prims[0])); q["p"][4] = th
                        cands.append(fit_group(tgt, win, [q], w, False))
                    prims = min(cands, key=lambda z: z[2])[0]
                else:
                    prims = fit_group(tgt, win, prims, w, False)[0]
                for extra in kinds[1:]:
                    blobs = residual_blobs(tgt, win, prims, w)
                    if not blobs:
                        continue
                    bx, by = comp_xy(blobs[0], win)
                    if extra == "dot":
                        prims.append({"t": "dot", "p": [bx.mean(), by.mean(), max(2.0, math.sqrt(len(bx) / S / S / math.pi))]})
                    else:
                        prims.append(init("line", bx, by, w)); prims[-1]["p"][4] = w
                ps, ww, fun = fit_group(tgt, win, prims, w, pass_ == 1)
                if ps[0]["t"] not in ("line",):
                    widths.append(ww)
                first.setdefault(k, []).append(ps)
                pivots.append([round(float(x.mean()), 1), round(float(y.mean()), 1)])
                groups.append([{**p, "p": [round(float(v), 2) for v in p["p"]]} for p in ps])
                report.append((pass_, k, spec, round(fun, 1)))
            out.setdefault("faces", {})[str(k)] = {"name": NAMES[str(k)]["name"], "groups": groups,
                                                   "pivots": dict(zip(["ojo-i", "ojo-d", "boca"], pivots))}
            print(f"pase {pass_} cara {k:2d} cuerpo IoU {iou:.3f}", flush=True)
        if pass_ == 1:
            out["stroke"] = round(float(np.median(widths)), 2)
            print("grosor de trazo común:", out["stroke"], "(p10–p90", np.percentile(widths, [10, 90]).round(2), ")")
    out["source"] = "docs/img/eddieexpresiones*.png → tools/fit_faces.py (primitivas geométricas, trazo común)"
    (E.FACE / "faces.json").write_text(json.dumps({"stroke": out["stroke"], "source": out["source"], "faces": out["faces"]}, ensure_ascii=False, indent=1))
    json.dump(report, open("/dev/stderr", "w"))


if __name__ == "__main__":
    main()
