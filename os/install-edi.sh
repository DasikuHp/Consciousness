#!/bin/bash
# Instala EDI en Omarchy con Niri (Okimarchy) desde este repo.
#   1) Arch + Okimarchy:  curl -fsSL https://raw.githubusercontent.com/cristian-fleischer/okimarchy/master/boot.sh | bash
#   2) Este script:       ./os/install-edi.sh
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
[ -f "$REPO/data/brain/flywire783/meta.json" ] || {
  echo ":: Preparando el cerebro (conectoma FlyWire v783)…"
  python3 "$REPO/tools/export_connectome.py"
}
sudo pacman -S --needed --noconfirm base-devel cargo jq curl
cd "$REPO/os"
EDI_SRC="$REPO" makepkg -f --noconfirm
sudo pacman -U --noconfirm edi-*.pkg.tar.zst
systemctl --user daemon-reload
systemctl --user enable --now edid.service
edi-niri-apply
edi status
