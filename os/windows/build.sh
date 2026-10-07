#!/bin/bash
# Compila EDI para Windows (x86_64-pc-windows-gnu, mingw) y arma dist/EDI-windows.zip
# Requisitos (Linux): rustup target add x86_64-pc-windows-gnu; apt install mingw-w64 zip
set -euo pipefail
R="$(cd "$(dirname "$0")/../.." && pwd)"
D="$R/dist/EDI-windows"
( cd "$R/edi" && cargo build --release --target x86_64-pc-windows-gnu -p edid -p edi-brain )
T="$R/edi/target/x86_64-pc-windows-gnu/release"
rm -rf "$D" && mkdir -p "$D/brain" "$D/face"
cp "$T/edid.exe" "$D/"; cp "$T/validate.exe" "$D/edi-brain-validate.exe"
cp "$R"/os/windows/{EDI.cmd,edi.cmd,autoinicio.cmd,LEEME.txt} "$D/"
for f in "$D"/*.cmd "$D/LEEME.txt"; do sed -i 's/$/\r/' "$f"; done   # CRLF
B="$R/data/brain/flywire783"
cp "$B"/{meta.json,region.u8,row_ptr.u32,col.u32,w.i32,root_id.i64,pos.f32,kc_thr_offset.f32,skel.json,skel_small.i16,skel_small.u32,skel_full.i16,skel_full.u32} "$D/brain/"
cp "$R"/os/assets/face/* "$D/face/"
( cd "$R/dist" && rm -f EDI-windows.zip && zip -qr9 EDI-windows.zip EDI-windows )
ls -la "$R/dist/EDI-windows.zip"
