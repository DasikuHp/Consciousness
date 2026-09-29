#!/bin/bash
# Fase 4: Connectome-RWKV v0 — conectoma real vs. barajado vs. sin cerebro (bypass)
cd "$(dirname "$0")/../.."
mkdir -p experiments/phase4/v1
for m in real shuffle bypass; do
  ./edi/target/release/edi-lang data/brain/flywire783 experiments/phase4/v1/$m $m 600 1 > experiments/phase4/v1/$m.out 2>&1
done
touch experiments/phase4/v1/DONE
