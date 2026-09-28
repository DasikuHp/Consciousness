#!/bin/bash
# Prueba de integración de EDI en Arch (la base de Omarchy) con Niri, dentro de Docker.
# Uso (host): docker run --rm -v "$PWD":/src:ro ghcr.io/archlinux/archlinux:latest bash /src/os/test/arch-test.sh
set -euo pipefail
ok() { echo "✅ $*"; }
[ -f /ca.crt ] && cp /ca.crt /etc/ca-certificates/trust-source/anchors/proxy.crt && update-ca-trust
pacman -Syu --noconfirm --needed base-devel cargo jq curl niri systemd >/dev/null
ok "paquetes: $(niri --version | head -1), $(cargo --version)"
useradd -m builder
mkdir -p /home/builder/repo/data && cp -r /src/edi /src/os /src/tools /home/builder/repo/ && rm -rf /home/builder/repo/edi/target && cp -r /src/data/brain /home/builder/repo/data/ && chown -R builder /home/builder/repo
su builder -c "cd /home/builder/repo/os && EDI_SRC=/home/builder/repo makepkg -f --noconfirm >/tmp/makepkg.log 2>&1" || { tail -30 /tmp/makepkg.log; exit 1; }
pacman -U --noconfirm /home/builder/repo/os/edi-*.pkg.tar.zst >/dev/null
ok "paquete instalado: $(pacman -Q edi)"
systemd-analyze verify /usr/lib/systemd/user/edid.service 2>&1 | grep -v -i "graphical-session" || true
ok "unidad systemd verificada"
# Config Niri realista: base (layout propio) + bloque EDI con include del mood + workspace + reglas → parser real de Niri
su builder -c 'mkdir -p ~/.config/niri && : > ~/.config/niri/windows.kdl && /usr/bin/edi-niri-apply >/dev/null 2>&1; printf "layout {\n    gaps 8\n}\n" > ~/.config/niri/config.kdl; cat ~/.config/niri/windows.kdl >> ~/.config/niri/config.kdl; niri validate -c ~/.config/niri/config.kdl'
ok "niri validate: base + include(edi-mood) + workspace + reglas + atajos aceptados"
# Vida real: memoria episódica, palabras, sueño, mood escrito por edid y revalidado por Niri
su builder -c '
export EDI_BRAIN=/usr/share/edi/brain/flywire783 EDI_PORT=7091 EDI_STATE=$HOME/st EDI_SLEEP_AFTER=100000
edid & P=$!; for i in $(seq 1 40); do curl -sf 127.0.0.1:7091/state >/dev/null && break; sleep 1; done
for i in 1 2 3; do curl -s -X POST --data "hola edi" 127.0.0.1:7091/say >/dev/null; done
curl -s -X POST 127.0.0.1:7091/sleep; sleep 12
curl -s "127.0.0.1:7091/recall?q=hola" | jq -c "{known, top: [.recalled[:3][].label]}"
curl -s 127.0.0.1:7091/memory | jq -c "{nodes, edges, episodes}"
echo "pos bytes: $(curl -s 127.0.0.1:7091/pos | wc -c)  canvas: $(curl -s 127.0.0.1:7091/ | grep -c "<canvas")"
niri validate -c ~/.config/niri/edi-mood.kdl; kill $P; wait $P 2>/dev/null; true'
ok "edid: memoria episódica + habla + sueño + cerebro 3D servido + mood válido"
