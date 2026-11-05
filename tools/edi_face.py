"""EDI en vector paramétrico: geometría exacta (no calco) + 30 expresiones como primitivas.

Fuente de verdad: os/assets/face/faces.json (rasgos de cada cara, ajustados a los diseños
docs/img/eddieexpresiones*.png por tools/fit_faces.py). Este script solo genera los SVG:
    python3 tools/edi_face.py            → os/assets/face/NN_nombre.svg (+ edi.svg, sprite)
Coordenadas en px del diseño original, y hacia abajo; figura de 200 × 311.
Las piezas llevan id (#cuerpo, #ojo-i, #ojo-d, #boca) para animarlas (parpadeo, hablar).
"""
import json, math
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FACE = ROOT / "os/assets/face"

# --- cuerpo (medido sobre la media de las 30 figuras; error del arco < 0,5 px) ---
T = 25.0          # grosor de arco, barra izquierda y base
CX, CY = 100.0, 100.0   # centro del arco
RO, RI = 100.0, 75.0    # radios exterior e interior
MID = (184.5, 206.7, 171.5)   # barra central: y0, y1, x derecha
BASE = (286.6, 311.0, 192.6)  # base: y0, y1, x derecha
YW, YS = 10.0, 11.5           # grosor de los brazos de la Y y de su tallo
YTOP, YJ = (59.3, 140.7), 262.0  # brazos: x en la barra central; unión en y
INK, CARD = "#111", "#f6f6f2"
PAD = 6.0
VB = (-PAD, -PAD, 200 + 2 * PAD, 311 + 2 * PAD)


def f(v):
    return f"{v:.2f}".rstrip("0").rstrip(".")


def body_path():
    m0, m1, mx = MID
    b0, b1, bx = BASE
    return (f"M0,{f(b1)}V{f(CY)}A{f(RO)},{f(RO)} 0 0 1 {f(CX + RO)},{f(CY)}H{f(CX + RI)}"
            f"A{f(RI)},{f(RI)} 0 0 0 {f(CX - RI)},{f(CY)}V{f(m0)}H{f(mx)}V{f(m1)}H{f(T)}"
            f"V{f(b0)}H{f(bx)}V{f(b1)}Z")


def y_path():
    """La Y como trazos: brazos (YW) hasta la unión y tallo (YS) hasta la base."""
    m1, b0 = MID[1], BASE[0]
    k = (CX - YTOP[0]) / (YJ - m1)  # pendiente de los brazos; empiezan 4 px dentro de la barra
    xl, xr = YTOP[0] - 4 * k, YTOP[1] + 4 * k
    return (f'<path d="M{f(xl)},{f(m1 - 4)}L{f(CX)},{f(YJ)}L{f(xr)},{f(m1 - 4)}" fill="none" stroke-width="{f(YW)}" stroke-linejoin="miter" stroke-miterlimit="20"/>'
            f'<path d="M{f(CX)},{f(YJ - 6)}V{f(b0 + 2)}" fill="none" stroke-width="{f(YS)}"/>')


def card_path():
    return f"M0,{f(BASE[1])}V{f(CY)}A{f(RO)},{f(RO)} 0 0 1 {f(CX + RO)},{f(CY)}V{f(BASE[1])}Z"


# --- primitivas de los rasgos ---
def rot(cx, cy, th):
    c, s = math.cos(math.radians(th)), math.sin(math.radians(th))
    return lambda u, v: (cx + u * c - v * s, cy + u * s + v * c)


def prim_svg(p, w):
    t, a = p["t"], p["p"]
    S = f'fill="none" stroke-width="{f(w)}"'
    if t == "semi":  # media elipse cerrada: cuerda + arco (dir 1 = hacia abajo)
        cx, cy, ra, rb, th = a
        P = rot(cx, cy, th)
        x0, y0 = P(-ra, 0); x1, y1 = P(ra, 0)
        sw = 1 if p["dir"] > 0 else 0
        return f'<path d="M{f(x0)},{f(y0)}L{f(x1)},{f(y1)}A{f(ra)},{f(rb)} {f(th)} 0 {sw} {f(x0)},{f(y0)}Z" {S} stroke-linejoin="miter" stroke-miterlimit="20"/>'
    if t == "ell":
        cx, cy, ra, rb, th = a
        return f'<ellipse cx="{f(cx)}" cy="{f(cy)}" rx="{f(ra)}" ry="{f(rb)}" transform="rotate({f(th)} {f(cx)} {f(cy)})" {S}/>'
    if t == "arc":  # arco abierto (sonrisa, ceño, ojo cerrado), ángulos paramétricos en grados
        cx, cy, ra, rb, th, t0, t1 = a
        P = rot(cx, cy, th)
        e = lambda tt: P(ra * math.cos(math.radians(tt)), rb * math.sin(math.radians(tt)))
        (x0, y0), (x1, y1) = e(t0), e(t1)
        big = 1 if abs(t1 - t0) > 180 else 0
        return f'<path d="M{f(x0)},{f(y0)}A{f(ra)},{f(rb)} {f(th)} {big} {1 if t1 > t0 else 0} {f(x1)},{f(y1)}" {S}/>'
    if t == "line":
        x0, y0, x1, y1, lw = a
        return f'<path d="M{f(x0)},{f(y0)}L{f(x1)},{f(y1)}" fill="none" stroke-width="{f(lw)}"/>'
    if t == "dot":
        cx, cy, r = a
        return f'<circle cx="{f(cx)}" cy="{f(cy)}" r="{f(r)}" stroke="none"/>'
    if t == "chev":
        x0, y0, x1, y1, x2, y2 = a
        return f'<path d="M{f(x0)},{f(y0)}L{f(x1)},{f(y1)}L{f(x2)},{f(y2)}" {S} stroke-linejoin="miter" stroke-miterlimit="20"/>'
    if t == "quad":  # cuadrilátero con esquinas redondeadas opcionales (radios r0..r3)
        pts = [(a[2 * i], a[2 * i + 1]) for i in range(4)]
        rs = a[8:12] if len(a) >= 12 else [0, 0, 0, 0]
        return f'<path d="{rounded(pts, rs)}" {S} stroke-linejoin="miter" stroke-miterlimit="20"/>'
    raise ValueError(t)


def fillet(pts, rs):
    """Esquinas redondeadas: para cada vértice, (punto de entrada, punto de salida, radio, giro)."""
    n, out = len(pts), []
    for i in range(n):
        p0, p1, p2 = pts[i - 1], pts[i], pts[(i + 1) % n]
        u = (p1[0] - p0[0], p1[1] - p0[1]); v = (p2[0] - p1[0], p2[1] - p1[1])
        lu, lv = math.hypot(*u), math.hypot(*v)
        u = (u[0] / lu, u[1] / lu); v = (v[0] / lv, v[1] / lv)
        cosang = max(-1.0, min(1.0, -(u[0] * v[0] + u[1] * v[1])))
        phi = math.acos(cosang)  # ángulo interior
        r = max(0.0, rs[i])
        d = min(r / math.tan(phi / 2) if phi > 1e-3 else 0, 0.45 * lu, 0.45 * lv) if r > 0 else 0
        r = d * math.tan(phi / 2) if d > 0 else 0
        a = (p1[0] - u[0] * d, p1[1] - u[1] * d); b = (p1[0] + v[0] * d, p1[1] + v[1] * d)
        out.append((a, b, r, 1 if u[0] * v[1] - u[1] * v[0] > 0 else 0))
    return out


def rounded(pts, rs):
    fl = fillet(pts, rs)
    d = f"M{f(fl[0][1][0])},{f(fl[0][1][1])}"
    for a, b, r, sw in fl[1:] + fl[:1]:
        d += f"L{f(a[0])},{f(a[1])}"
        if r > 0:
            d += f"A{f(r)},{f(r)} 0 0 {sw} {f(b[0])},{f(b[1])}"
    return d + "Z"


def svg(face, w, card=True, size=None):
    parts = {"ojo-i": [], "ojo-d": [], "boca": []}
    for g, prims in zip(["ojo-i", "ojo-d", "boca"], face["groups"]):
        parts[g] = [prim_svg(p, w) for p in prims]
    vb = " ".join(f(v) for v in VB)
    wh = f' width="{size[0]}" height="{size[1]}"' if size else ""
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}"{wh}>',
           f'<title>EDI · {face["name"].replace("_", " ")}</title>']
    if card:
        out.append(f'<path id="tarjeta" fill="{CARD}" stroke="{CARD}" stroke-width="{f(2 * PAD - 2)}" stroke-linejoin="round" d="{card_path()}"/>')
    out.append(f'<g id="cuerpo" fill="{INK}" stroke="{INK}" stroke-linecap="butt"><path stroke="none" d="{body_path()}"/>{y_path()}</g>')
    for g, el in parts.items():
        cx, cy = face.get("pivots", {}).get(g, (100, 120))
        out.append(f'<g id="{g}" fill="{INK}" stroke="{INK}" stroke-linecap="butt" style="transform-origin:{f(cx)}px {f(cy)}px">{"".join(el)}</g>')
    out.append("</svg>")
    return "".join(out)


def main():
    spec = json.loads((FACE / "faces.json").read_text())
    w = spec["stroke"]
    for k, face in spec["faces"].items():
        (FACE / f"{int(k):02d}_{face['name']}.svg").write_text(svg(face, w))
    print(f"{len(spec['faces'])} caras SVG (trazo {w})")


if __name__ == "__main__":
    main()
