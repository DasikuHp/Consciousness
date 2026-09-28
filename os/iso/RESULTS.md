# ISO EDI.os — resultado de la construcción (2026-09-28)

- `mkarchiso` (perfil baseline de archiso + Niri + paquete `edi` desde repo pacman local) → `edi-os-2026.09.28-x86_64.iso`, **758 MB**, arranque BIOS y UEFI (El Torito).
- Verificado dentro del sistema de ficheros de la ISO: `edid`, `edi`, `edi-shell`, `niri`, el cerebro (`meta.json`, `pos.f32`…), el bloque Niri, `edid-live.service` habilitado y el motd.
- **No arrancada:** esta nube no tiene KVM ni QEMU ni monta ISO. Falta probar el arranque en una VM o en el portátil.
- Se genera con `os/iso/build.sh` (Docker `--privileged`); la ISO no se sube al repo (`out/` ignorado).
