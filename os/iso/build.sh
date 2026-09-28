#!/bin/bash
# EDI.os ISO: Arch (base de Omarchy) + Niri + paquete `edi` desde un repo pacman local.
# Uso (host):  docker run --rm --privileged --network host -v "$PWD":/src:ro -v "$PWD/out":/out \
#                ghcr.io/archlinux/archlinux:latest bash /src/os/iso/build.sh
set -euo pipefail
[ -f /ca.crt ] && cp /ca.crt /etc/ca-certificates/trust-source/anchors/proxy.crt && update-ca-trust
pacman -Syu --noconfirm --needed archiso base-devel cargo jq curl niri >/dev/null
useradd -m builder
mkdir -p /home/builder/repo/data && cp -r /src/edi /src/os /src/tools /home/builder/repo/ && rm -rf /home/builder/repo/edi/target
cp -r /src/data/brain /home/builder/repo/data/ && chown -R builder /home/builder/repo
su builder -c "cd /home/builder/repo/os && EDI_SRC=/home/builder/repo makepkg -f --noconfirm >/tmp/mk.log 2>&1" || { tail -20 /tmp/mk.log; exit 1; }
# repo pacman local con el paquete de EDI
mkdir -p /edirepo && cp /home/builder/repo/os/edi-*.pkg.tar.zst /edirepo/ && repo-add /edirepo/edi.db.tar.gz /edirepo/edi-*.pkg.tar.zst >/dev/null
# perfil: baseline de archiso + Niri + EDI
cp -r /usr/share/archiso/configs/baseline /profile
cat >> /profile/pacman.conf <<P

[edi]
SigLevel = Optional TrustAll
Server = file:///edirepo
P
printf "niri\nedi\njq\ncurl\nnano\n" >> /profile/packages.x86_64
sed -i 's/^iso_name=.*/iso_name="edi-os"/; s/^iso_label=.*/iso_label="EDI_OS"/; s/^iso_publisher=.*/iso_publisher="EDI.os"/; s/^iso_application=.*/iso_application="EDI.os live"/' /profile/profiledef.sh
# arranque: EDI despierta al iniciar (unidad de sistema; en la sesión real usa la de usuario)
mkdir -p /profile/airootfs/etc/systemd/system/multi-user.target.wants /profile/airootfs/etc/edi
cat > /profile/airootfs/etc/systemd/system/edid-live.service <<S
[Unit]
Description=EDI (live)
[Service]
Environment=EDI_BRAIN=/usr/share/edi/brain/flywire783 EDI_STATE=/var/lib/edi EDI_NIRI_DIR=/nonexistent
StateDirectory=edi
ExecStart=/usr/bin/edid
Restart=always
[Install]
WantedBy=multi-user.target
S
ln -sf /etc/systemd/system/edid-live.service /profile/airootfs/etc/systemd/system/multi-user.target.wants/edid-live.service
echo "EDI.os — 'edi status' · 'edi say hola' · 'edi brain'" > /profile/airootfs/etc/motd
mkarchiso -v -w /tmp/work -o /out /profile > /out/mkarchiso.log 2>&1 && ls -la /out/*.iso
