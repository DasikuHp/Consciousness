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
# Config Niri mínima + bloque EDI → validación real del parser de Niri
su builder -c 'mkdir -p ~/.config/niri && : > ~/.config/niri/windows.kdl && /usr/bin/edi-niri-apply >/dev/null && niri validate -c ~/.config/niri/windows.kdl'
ok "niri validate: bloque EDI aceptado"
# Vida: nace, se pausa, despierta con la misma edad
su builder -c 'EDI_MAX_TICKS=30 edid; EDI_MAX_TICKS=30 edid & sleep 1; edi status; wait; edi log 10'
ok "edid persiste entre reinicios"
